#![allow(unused_variables)]
#![allow(dead_code)]

use iced::{
    widget::{button, column, text},
    time,
    Element, Subscription, Task,
};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};

use crate::api::{C, S, B, ServerMessage};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;

#[derive(Debug)]
struct App {
    started: bool,
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    last_msg: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            started: false,
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
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
    TestBid,
    TestPlayCard,
    Tick,
}

fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    match msg {
        AppMessage::Start => {
            // Start server first
            if !state.started {
                let _ = crate::server::start_server();
                state.started = true;
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            
            // Then connect to it
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        connect_ws(ws_tx, server_rx).await;
                    });
                });
                state.connected = true;
                state.last_msg = "Connecting...".to_string();
            }
            Task::none()
        }
        AppMessage::Connect => {
            // connect to localhost:3000 without starting a server
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        connect_ws(ws_tx, server_rx).await;
                    });
                });
                state.connected = true;
                state.last_msg = "Connecting...".to_string();
            }
            Task::none()
        }
        AppMessage::JoinLobby => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby { name: "Player".to_string() });
                    state.last_msg = "Joining lobby...".to_string();
                }
            }
            Task::none()
        }
        AppMessage::LeaveLobby => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::LeaveLobby);
                    state.last_msg = "Leaving lobby...".to_string();
                }
            }
            Task::none()
        }
        AppMessage::ToggleReady => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetReady { ready: true });
                    state.last_msg = "Set ready".to_string();
                }
            }
            Task::none()
        }
        AppMessage::TestBid => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::Bid { amount: 3 });
                    state.last_msg = "Bid 3".to_string();
                }
            }
            Task::none()
        }
        AppMessage::TestPlayCard => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    use crate::api::{Card, Suit, Value};
                    let _ = tx.send(C::PlayCard { 
                        card: Card { value: Value::Number(5), suit: Suit::Red } 
                    });
                    state.last_msg = "Played card".to_string();
                }
            }
            Task::none()
        }
        AppMessage::Tick => {
            // Check for messages from server
            if let Ok(guard) = state.server_rx.lock() {
                if let Some(ref rx) = *guard {
                    while let Ok(msg) = rx.try_recv() {
                        println!("Received: {:?}", msg);
                        match msg {
                            ServerMessage::Server(s) => match s {
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
                            ServerMessage::Broadcast(b) => match b {
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
            }
            Task::none()
        }
    }
}

async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver) {
    println!("Attempting to connect...");
    match connect_async("ws://127.0.0.1:3000/ws").await {
        Ok((ws_stream, _)) => {
            println!("WebSocket connected!");
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (srv_tx, srv_rx) = std::sync::mpsc::channel();

            *ws_tx.lock().unwrap() = Some(tx);
            *server_rx.lock().unwrap() = Some(srv_rx);
            println!("Receiver set successfully!");

            // Send task
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if write.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
            });

            // Receive task - parse as ServerMessage directly
            let recv_task = tokio::spawn(async move {
                while let Some(Ok(WsMessage::Text(txt))) = read.next().await {
                    println!("Raw message received: {}", txt);
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        println!("Parsed successfully: {:?}", server_msg);
                        let _ = srv_tx.send(server_msg);
                    } else {
                        println!("Failed to parse message");
                    }
                }
                println!("Receive loop ended");
            });

            // Wait for either task to complete
            tokio::select! {
                _ = send_task => println!("Send task ended"),
                _ = recv_task => println!("Receive task ended"),
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
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
        button("Test Bid (3)").on_press(AppMessage::TestBid),
        button("Test Play Card").on_press(AppMessage::TestPlayCard),
        text(format!("Status: {}", state.last_msg)),
    ]
    .padding(20)
    .into()
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    // Use iced's time::every for polling - this is the correct way
    if state.connected {
        time::every(Duration::from_millis(100)).map(|_| AppMessage::Tick)
    } else {
        Subscription::none()
    }
}

pub fn main() -> iced::Result {
    iced::application("Wizard", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(300.0, 300.0),
            resizable: true,
            ..Default::default()
        })
        .run()
}
