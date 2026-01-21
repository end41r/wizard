use crate::api::{B, C, Lobby, Player, S, ServerMessage};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

type Clients = Arc<RwLock<HashMap<u64, mpsc::UnboundedSender<ServerMessage>>>>;
type PlayerList = Arc<RwLock<HashMap<u64, Player>>>;

const SVERSION: usize = 1;

// evade unsafe static mut
static MAX_PLAYER_COUNT: AtomicUsize = AtomicUsize::new(4);

static SHUTDOWN_SENDER: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
pub fn local_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| {
            s.connect("8.8.8.8:80").ok()?;
            s.local_addr().ok()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or("?".into())
}

pub fn start_server() {
    // create the shared state up-front so we can expose it to `stop_server()`
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let players: PlayerList = Arc::new(RwLock::new(HashMap::new()));

    // store globals for `stop_server` to use
    // create a oneshot channel that will be used to trigger graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    // store the sender in the global so stop_server() can use it
    if let Ok(mut guard) = SHUTDOWN_SENDER.lock() {
        *guard = Some(tx);
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            run_server(clients, players, rx).await;
        });
    });
}

async fn run_server(clients: Clients, players: PlayerList, shutdown_rx: tokio::sync::oneshot::Receiver<()>) {
    let app = Router::new().route(
        "/ws",
        get({
            let clients = clients.clone();
            let players = players.clone();
            move |ws| ws_handler(ws, clients.clone(), players.clone())
        }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("server on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tokio::select! {
        res = axum::serve(listener, app) => {
            if let Err(e) = res {
                eprintln!("server error: {e}");
            }
        }
        _ = shutdown_rx => {
            println!("Shutdown signal received, broadcasting shutdown and stopping server");
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, clients: Clients, players: PlayerList) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients, players))
}

// part with shutting down was made with help from chatgpt
pub fn stop_server() {
    if let Ok(mut guard) = SHUTDOWN_SENDER.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }
}

// Broadcast to all connected clients.
async fn broadcast_to_all(clients: &Clients, _players: &PlayerList, broadcast: B) {
    let wrapped = ServerMessage::Broadcast(broadcast);
    let message_text = serde_json::to_string(&wrapped).unwrap();
    println!("Broadcasting: {message_text}");
    let clients_map = clients.read().await;
    for (client_id, tx) in clients_map.iter() {
        if tx.send(wrapped.clone()).is_err() {
            println!("Failed to send to client {client_id}");
        }
    }
}

async fn handle_socket(socket: WebSocket, clients: Clients, players: PlayerList) {
    let id = Uuid::new_v4().as_u128() as u64;
    println!("New connection: player {id}");

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Register the client.
    clients.write().await.insert(id, tx.clone());


    // Spawn task to forward messages from channel to websocket.
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if sender.send(Message::Text(text)).await.is_err() {
                        println!("Failed to send to player {id}");
                        break;
                    }
                }
                Err(e) => {
                    println!("Serialization error for player {id}: {e}");
                    break;
                }
            }
        }
    });

    // Run the main receive loop.
    let clients_clone = clients.clone();
    let players_clone = players.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<C>(&text) {
                    Ok(C::Handshake { version }) => {
                        println!("Received handshake with version: {version}");
                        let response = ServerMessage::Server(S::HandshakeConfirmation {
                            version: SVERSION,
                            supported: version == SVERSION,
                        });
                        if let Err(e) = tx.send(response) {
                            println!("Failed to send handshake confirmation: {e:?}");
                        }
                    }
                    Ok(C::JoinLobby { name }) => {
                        println!("Player {id} trying to join lobby as {name}");
                        let players_map = players_clone.read().await;

                        if players_map.len() >= MAX_PLAYER_COUNT.load(Ordering::Relaxed) {
                            let error = ServerMessage::Server(S::Error {
                                reason: "Lobby is full".to_string(),
                            });
                            if let Err(e) = tx.send(error) {
                                println!("Failed to send error: {e:?}");
                            }
                            return;
                        }
                        else if players_map.values().any(|p| p.name == name) {
                            let error = ServerMessage::Server(S::Error {
                                reason: "Name already taken".to_string(),
                            });
                            if let Err(e) = tx.send(error) {
                                println!("Failed to send error: {e:?}");
                            }
                            return;
                        }
                        else {
                            let response = ServerMessage::Server(S::JoinConfirmation { ok: true, id: id });
                            if let Err(e) = tx.send(response) {
                                println!("Failed to send join confirmation: {e:?}");
                            }
                        }
                        drop(players_map);
                        // Add player to the players list
                        let is_host = players_clone.read().await.is_empty(); // thats really unsafe
                        let player = Player {
                            id,
                            name: name.clone(),
                            ready: false,
                            is_host,
                        };
                        players_clone.write().await.insert(id, player);

                        // Broadcast the updated lobby state.
                        let players_list: Vec<Player> = players_clone
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect();
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::LobbyState {
                                lobby: Some(Lobby {
                                    players: players_list,
                                    chat: vec![],
                                }),

                            },
                        ).await;
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::PlayerCountChanged { count: MAX_PLAYER_COUNT.load(Ordering::Relaxed) },
                        )
                        .await;
                    }
                    Ok(C::LeaveLobby) => {
                        println!("Player {id} leaving lobby");

                        // Remove player from the players list
                        players_clone.write().await.remove(&id);

                        // Broadcast the updated lobby state.
                        let players_list: Vec<Player> = players_clone
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect();
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::LobbyState {
                                lobby: Some(Lobby {
                                    players: players_list,
                                    chat: vec![],
                                }),
                            },
                        )
                        .await;
                    }
                    Ok(C::ChatMessage { sender, message }) => {
                        println!("Player {sender} sent chat message: {message}");

                        // Broadcast the chat message.
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::ChatMessage {
                                sender: players_clone
                                    .read()
                                    .await
                                    .get(&id)
                                    .map(|p| p.id)
                                    .unwrap_or(0),
                                message: message.clone(),
                            },
                        )
                        .await;
                    }
                    Ok(C::SetReady { ready }) => {
                        println!("Player {id} set ready: {ready}");

                        // Update player ready status
                        if let Some(player) = players_clone.write().await.get_mut(&id) {
                            player.ready = ready;
                        }

                        // Broadcast the updated lobby state.
                        let players_list: Vec<Player> = players_clone
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect();
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::LobbyState {
                                lobby: Some(Lobby {
                                    players: players_list,
                                    chat: vec![],
                                }),
                            },
                        )
                        .await;
                    }
                    Ok(C::Bid { amount }) => {
                        println!("Player {id} bid: {amount}");

                        // Broadcast the bid.
                        broadcast_to_all(&clients_clone, &players_clone, B::BidMade { player: id, amount }).await;
                        // TODO: Validate the bid and check if the bidding is complete.
                    }
                    Ok(C::PlayCard { card }) => {
                        println!("Player {id} played card: {card:?}");

                        // Broadcast the played card.
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::CardPlayed {
                                player: id,
                                card: card.clone(),
                            },
                        )
                        .await;
                        // TODO: Validate the play and check if the pool is complete.
                    }
                    Ok(C::StartGame) => {
                        println!("Player {id} requested to start game");
                        let players_list: Vec<Player> = players_clone
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect();
                        
                        // Broadcast game started to all players
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::GameStarted {
                                players: players_list.iter().map(|p| p.id).collect(),
                            },
                        )
                        .await;
                    }
                    Ok(C::SetPlayerCount { count }) => {
                        println!("Player {id} set player count to {count}");
                        MAX_PLAYER_COUNT.store(count, Ordering::Relaxed);
                        // Broadcast player count change to all players
                        broadcast_to_all(
                            &clients_clone,
                            &players_clone,
                            B::PlayerCountChanged { count },
                        )
                        .await;
                    }
                    Ok(C::RequestShutdown) => {
                        println!("Player {id} requested server shutdown");
                        // Best-effort broadcast shutdown to all connected clients before stopping server.
                        broadcast_to_all(&clients_clone, &players_clone, B::ServerShutdown).await;
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        stop_server();
                    }
                    Err(err) => {
                        println!("Parse error from player {id}: {err}");
                        let error = ServerMessage::Server(S::Error {
                            reason: format!("Invalid message: {err}"),
                        });
                        let _ = tx.send(error);
                    }
                }
            }
        }
    });


    // Wait for either task to finish.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }


    // Cleanup: Remove the client.
    clients.write().await.remove(&id);
    println!("Player {id} disconnected");
}
