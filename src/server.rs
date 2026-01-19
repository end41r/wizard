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
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

type Clients = Arc<RwLock<HashMap<u64, mpsc::UnboundedSender<ServerMessage>>>>;
type PlayerList = Arc<RwLock<HashMap<u64, Player>>>;

const SVERSION: usize = 1;

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
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
            let players: PlayerList = Arc::new(RwLock::new(HashMap::new()));
            run_server(clients, players).await;
        });
    });
}

async fn run_server(clients: Clients, players: PlayerList) {
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
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, clients: Clients, players: PlayerList) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients, players))
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
                        println!("Player {id} joining lobby as {name}");
                        let response = ServerMessage::Server(S::JoinConfirmation { ok: true });
                        if let Err(e) = tx.send(response) {
                            println!("Failed to send join confirmation: {e:?}");
                        }

                        // Add player to the players list
                        let is_host = players_clone.read().await.is_empty();
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
                                sender: id,
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
