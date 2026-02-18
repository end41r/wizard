use crate::api::{AvatarKind, Lobby, Player, PlayerId, ServerMessage, B, C, S};
use crate::gamelogic::game::Game;
use crate::gamelogic::GameEvent;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

type Clients = Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>;
type PlayerList = Arc<RwLock<HashMap<PlayerId, Player>>>;
type SharedGame = Arc<RwLock<Game>>;

const SVERSION: usize = 1;

// Uses an atomic to avoid unsafe static mut use.
static MAX_PLAYER_COUNT: AtomicUsize = AtomicUsize::new(4);

static SHUTDOWN_SENDER: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
static CLIENTS: Mutex<Option<Clients>> = Mutex::new(None);

/// Game events -> Network messages
async fn dispatch_events(events: Vec<GameEvent>, clients: &Clients, players: &PlayerList) {
    for event in events {
        use GameEvent::*;
        match event {
            GameStarted { players: p } => {
                broadcast(clients, players, B::GameStarted { players: p }).await
            }
            GameFinished {
                final_scores,
                winner,
            } => {
                let final_scores: Vec<_> = final_scores.into_iter().collect();
                broadcast(
                    clients,
                    players,
                    B::GameFinished {
                        final_scores,
                        winner,
                    },
                )
                .await;
            }
            RoundStarted {
                round,
                cards_per_player,
                trump,
            } => {
                broadcast(
                    clients,
                    players,
                    B::RoundStarted {
                        round,
                        cards_per_player,
                        trump,
                    },
                )
                .await
            }
            RoundFinished { scores, tricks_won } => {
                let scores: Vec<_> = scores.into_iter().collect();
                let won_amounts: Vec<_> = tricks_won.into_iter().collect();
                broadcast(
                    clients,
                    players,
                    B::RoundFinished {
                        scores,
                        won_amounts,
                    },
                )
                .await;
            }
            HandDealt { player, cards } => send(clients, player, S::HandDealt { cards }).await,
            DealerMustSetTrump { dealer } => {
                broadcast(clients, players, B::DealerMustSetTrump { dealer }).await;
                send(clients, dealer, S::TrumpRequest).await;
            }
            TrumpSet { suit, by_dealer } => {
                broadcast(clients, players, B::TrumpSet { suit, by_dealer }).await
            }
            BiddingStarted {
                starting_player,
                cards_per_player,
            } => {
                broadcast(
                    clients,
                    players,
                    B::BiddingStarted {
                        starting_player,
                        cards_per_player,
                    },
                )
                .await
            }
            BidRequest { player, min, max } => {
                broadcast(clients, players, B::BidTurn { player }).await;
                send(clients, player, S::BidRequest { min, max }).await;
            }
            BidMade { player, amount } => {
                broadcast(clients, players, B::BidMade { player, amount }).await
            }
            BiddingFinished { bids } => {
                let bids: Vec<_> = bids.into_iter().collect();
                broadcast(clients, players, B::BiddingFinished { bids }).await
            }
            TrickStarted { leader } => broadcast(clients, players, B::PoolStarted { leader }).await,
            TurnRequest {
                player,
                valid_cards,
            } => {
                broadcast(clients, players, B::TurnChanged { player }).await;
                send(clients, player, S::YourTurn { valid_cards }).await;
            }
            CardPlayed { player, card } => {
                broadcast(clients, players, B::CardPlayed { player, card }).await
            }
            TrickFinished { winner, cards } => {
                broadcast(clients, players, B::PoolFinished { winner, cards }).await
            }
        }
    }
}

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
    let game: SharedGame = Arc::new(RwLock::new(Game::new()));

    // store clients globally so send() can access them
    if let Ok(mut guard) = CLIENTS.lock() {
        *guard = Some(clients.clone());
    }

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
            run_server(clients, players, game, rx).await;
        });
    });
}

async fn run_server(
    clients: Clients,
    players: PlayerList,
    game: SharedGame,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let app = Router::new().route(
        "/ws",
        get({
            let clients = clients.clone();
            let players = players.clone();
            let game = game.clone();
            move |ws| ws_handler(ws, clients.clone(), players.clone(), game.clone())
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    clients: Clients,
    players: PlayerList,
    game: SharedGame,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients, players, game))
}

/// Stops the server by sending a shutdown signal.
/// Was made using Claude Opuss' help.
pub fn stop_server() {
    if let Ok(mut guard) = SHUTDOWN_SENDER.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }
}

async fn broadcast(clients: &Clients, _players: &PlayerList, msg: B) {
    let wrapped = ServerMessage::Broadcast(msg);
    for tx in clients.read().await.values() {
        let _ = tx.send(wrapped.clone());
    }
}

async fn send(clients: &Clients, player_id: PlayerId, msg: S) {
    if let Some(tx) = clients.read().await.get(&player_id) {
        let _ = tx.send(ServerMessage::Server(msg));
    }
}

async fn handle_socket(socket: WebSocket, clients: Clients, players: PlayerList, game: SharedGame) {
    let id: PlayerId = Uuid::new_v4().as_u128() as u64;
    println!("New connection: player {id}");

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    clients.write().await.insert(id, tx.clone());

    // Spawns a task to forward messages from the channel to the WebSocket.
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

    // Runs the main receive loop.
    let clients_clone = clients.clone();
    let players_clone = players.clone();
    let game_clone = game.clone();
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
                        } else if players_map.values().any(|p| p.name == name) {
                            let error = ServerMessage::Server(S::Error {
                                reason: "Name already taken".to_string(),
                            });
                            if let Err(e) = tx.send(error) {
                                println!("Failed to send error: {e:?}");
                            }
                            return;
                        } else {
                            let response =
                                ServerMessage::Server(S::JoinConfirmation { ok: true, id });
                            if let Err(e) = tx.send(response) {
                                println!("Failed to send join confirmation: {e:?}");
                            }
                        }
                        drop(players_map);
                        let is_host = players_clone.read().await.is_empty(); // thats really unsafe
                        let player = Player {
                            id,
                            name: name.clone(),
                            avatar: AvatarKind::Mage,
                            ready: false,
                            is_host,
                        };
                        players_clone.write().await.insert(id, player);

                        let players_list: Vec<Player> =
                            players_clone.read().await.values().cloned().collect();

                        let res = game_clone.write().await.add_player(id);
                        if res.is_err() {
                            let error = ServerMessage::Server(S::Error {
                                reason: res.err().unwrap().to_string(),
                            });
                            if let Err(e) = tx.send(error) {
                                println!("Failed to send error: {e:?}");
                            }
                        }

                        broadcast(
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
                        broadcast(
                            &clients_clone,
                            &players_clone,
                            B::PlayerCountChanged {
                                count: MAX_PLAYER_COUNT.load(Ordering::Relaxed),
                            },
                        )
                        .await;
                    }
                    Ok(C::LeaveLobby) => {
                        println!("Player {id} leaving lobby");

                        players_clone.write().await.remove(&id);

                        let players_list: Vec<Player> =
                            players_clone.read().await.values().cloned().collect();

                        let res = game_clone.write().await.remove_player(id);
                        if res.is_err() {
                            let error = ServerMessage::Server(S::Error {
                                reason: res.err().unwrap().to_string(),
                            });
                            if let Err(e) = tx.send(error) {
                                println!("Failed to send error: {e:?}");
                            }
                            return;
                        }

                        broadcast(
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

                        broadcast(
                            &clients_clone,
                            &players_clone,
                            B::ChatMessage {
                                sender: players_clone
                                    .read()
                                    .await
                                    .get(&id)
                                    .map(|p| p.name.clone())
                                    .unwrap_or("Unknown".to_string()),
                                message: message.clone(),
                            },
                        )
                        .await;
                    }
                    Ok(C::SetReady { ready }) => {
                        println!("Player {id} set ready: {ready}");

                        if let Some(player) = players_clone.write().await.get_mut(&id) {
                            player.ready = ready;
                        }

                        let players_list: Vec<Player> =
                            players_clone.read().await.values().cloned().collect();
                        broadcast(
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
                        match game_clone.write().await.bid(id, amount) {
                            Err(e) => {
                                let _ = tx.send(ServerMessage::Server(S::Error {
                                    reason: e.to_string(),
                                }));
                            }
                            Ok(events) => {
                                dispatch_events(events, &clients_clone, &players_clone).await
                            }
                        }
                    }
                    Ok(C::PlayCard { card }) => {
                        println!("Player {id} played card: {card:?}");
                        match game_clone.write().await.play_card(id, card) {
                            Err(e) => {
                                let _ = tx.send(ServerMessage::Server(S::Error {
                                    reason: e.to_string(),
                                }));
                            }
                            Ok(events) => {
                                dispatch_events(events, &clients_clone, &players_clone).await
                            }
                        }
                    }
                    Ok(C::StartGame) => {
                        println!("Player {id} requested to start game");
                        match game_clone.write().await.start() {
                            Err(e) => {
                                let _ = tx.send(ServerMessage::Server(S::Error {
                                    reason: e.to_string(),
                                }));
                            }
                            Ok(events) => {
                                dispatch_events(events, &clients_clone, &players_clone).await
                            }
                        }
                    }
                    Ok(C::SetTrump { suit }) => {
                        println!("Player {id} setting trump to {suit:?}");
                        match game_clone.write().await.set_trump(id, suit) {
                            Err(e) => {
                                let _ = tx.send(ServerMessage::Server(S::Error {
                                    reason: e.to_string(),
                                }));
                            }
                            Ok(events) => {
                                dispatch_events(events, &clients_clone, &players_clone).await
                            }
                        }
                    }
                    Ok(C::SetPlayerCount { count }) => {
                        println!("Player {id} set player count to {count}");
                        MAX_PLAYER_COUNT.store(count, Ordering::Relaxed);

                        broadcast(
                            &clients_clone,
                            &players_clone,
                            B::PlayerCountChanged { count },
                        )
                        .await;
                    }
                    Ok(C::RequestShutdown) => {
                        println!("Player {id} requested server shutdown");
                        broadcast(&clients_clone, &players_clone, B::ServerShutdown).await;
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

    // Waits for either task to finish.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Removes the client from the shared map.
    clients.write().await.remove(&id);
    println!("Player {id} disconnected");
}
