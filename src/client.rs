#![allow(unused_variables)]
#![allow(dead_code)]

use iced::{
    widget::{button, column, text},
    Element, Subscription, Task,
};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};

use crate::api::{C, S, B};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<S>>>>;
type BroadcastReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<B>>>>;

#[derive(Debug)]
struct App {
    started: bool,
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    broadcast_rx: BroadcastReceiver,
    last_msg: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            started: false,
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            broadcast_rx: Arc::new(Mutex::new(None)),
            last_msg: "Not connected".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Start,
    Connect,
    JoinLobby,
    LeaveLobby,
    ToggleReady,
    CheckMessages,
}

fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    if !state.started {
        let _ = crate::server::start_server();
        state.started = true;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    match msg {
        AppMessage::Start => {
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                let broadcast_rx = Arc::clone(&state.broadcast_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        connect_ws(ws_tx, server_rx, broadcast_rx).await;
                    });
                });
                state.connected = true;
                state.last_msg = "Connecting...".to_string();
            }
            // Start checking messages
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::Connect => {
            // connect to localhost:3000 without starting a server
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                let broadcast_rx = Arc::clone(&state.broadcast_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        connect_ws(ws_tx, server_rx, broadcast_rx).await;
                    });
                });
                state.connected = true;
                state.last_msg = "Connecting...".to_string();
            }
            
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::JoinLobby => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby { name: "Player".to_string() });
                    state.last_msg = "Joining lobby...".to_string();
                }
            }
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::LeaveLobby => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::LeaveLobby);
                    state.last_msg = "Leaving lobby...".to_string();
                }
            }
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::ToggleReady => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetReady { ready: true });
                    state.last_msg = "Set ready".to_string();
                }
            }
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::CheckMessages => {
            if let Ok(guard) = state.server_rx.lock() {
                if let Some(ref rx) = *guard {
                    while let Ok(msg) = rx.try_recv() {
                        println!("Received: {:?}", msg);
                        match msg {
                            S::JoinConfirmation { ok } => {
                                state.last_msg = format!("Join: {}", ok);
                            }
                            S::Error { reason } => {
                                state.last_msg = format!("Error: {}", reason);
                            }
                            S::HandDealt { cards } => {
                                state.last_msg = format!("Hand dealt: {} cards", cards.len());
                            }
                            S::BidRequest { min, max } => {
                                state.last_msg = format!("Bid request: {}-{}", min, max);
                            }
                            S::InvalidBid { reason } => {
                                state.last_msg = format!("Invalid bid: {}", reason);
                            }
                            S::YourTurn { valid_cards } => {
                                state.last_msg = format!("Your turn: {} cards", valid_cards.len());
                            }
                            S::InvalidMove { reason } => {
                                state.last_msg = format!("Invalid move: {}", reason);
                            }
                       }
                    }
                } else {
                    println!("No receiver yet");
                }
            }
            
            // Check for broadcast messages
            if let Ok(guard) = state.broadcast_rx.lock() {
                if let Some(ref rx) = *guard {
                    while let Ok(broadcast) = rx.try_recv() {
                        println!("Broadcast: {:?}", broadcast);
                        match broadcast {
                            B::LobbyState { players } => {
                                state.last_msg = format!("Lobby: {} players", players.len());
                            }
                            B::GameStarted { players } => {
                                state.last_msg = format!("Game started: {} players", players.len());
                            }
                            B::RoundStarted { round, cards_per_player, trump } => {
                                state.last_msg = format!("Round {} started: {} cards", round, cards_per_player);
                            }
                            B::BiddingStarted { starting_player, cards_per_player } => {
                                state.last_msg = format!("Bidding started: {} cards", cards_per_player);
                            }
                            B::BidTurn { player } => {
                                state.last_msg = format!("Player {} bidding", player);
                            }
                            B::BidMade { player, amount } => {
                                state.last_msg = format!("Player {} bid {}", player, amount);
                            }
                            B::BiddingFinished { bids } => {
                                state.last_msg = format!("Bidding done: {} bids", bids.len());
                            }
                            B::PoolStarted { leader } => {
                                state.last_msg = format!("Pool started, leader: {}", leader);
                            }
                            B::TurnChanged { player } => {
                                state.last_msg = format!("Turn: player {}", player);
                            }
                            B::CardPlayed { player, card } => {
                                state.last_msg = format!("Player {} played card", player);
                            }
                            B::PoolFinished { winner, cards } => {
                                state.last_msg = format!("Pool won by {}", winner);
                            }
                            B::RoundFinished { scores, won_amounts } => {
                                state.last_msg = format!("Round finished");
                            }
                            B::GameFinished { final_scores, winner } => {
                                state.last_msg = format!("Game won by {}", winner);
                            }
                        }
                    }
                }
            }

            Task::future(async {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                AppMessage::CheckMessages
            })
        }
    }
}

async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver, broadcast_rx: BroadcastReceiver) {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:3000/ws").await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (srv_tx, srv_rx) = std::sync::mpsc::channel();
    let (bcast_tx, bcast_rx) = std::sync::mpsc::channel();

    *ws_tx.lock().unwrap() = Some(tx);
    *server_rx.lock().unwrap() = Some(srv_rx);
    *broadcast_rx.lock().unwrap() = Some(bcast_rx);

    // Send task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if write.send(WsMessage::Text(text)).await.is_err() {
                break;
            }
        }
    });

    // Receive task - try S first, then B
    while let Some(Ok(WsMessage::Text(txt))) = read.next().await {
        if let Ok(server_msg) = serde_json::from_str::<S>(&txt) {
            let _ = srv_tx.send(server_msg);
        } else if let Ok(broadcast_msg) = serde_json::from_str::<B>(&txt) {
            let _ = bcast_tx.send(broadcast_msg);
        }
    }
}

fn view(state: &'_ App) -> Element<'_, AppMessage> {
    column![
        button("Start").on_press(AppMessage::Start),
        button("Connect").on_press(AppMessage::Connect),
        button("Join Lobby").on_press(AppMessage::JoinLobby),
        button("Leave Lobby").on_press(AppMessage::LeaveLobby),
        button("Toggle Ready").on_press(AppMessage::ToggleReady),
        text(format!("Status: {}", state.last_msg)),
    ]
    .padding(20)
    .into()
}

fn subscription(_state: &App) -> Subscription<AppMessage> {
    Subscription::none()
}pub fn main() -> iced::Result {
    iced::application("Wizard", update, view)
        .subscription(subscription)
        .run()
}
