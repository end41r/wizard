//#![allow(unused_variables)]
//#![allow(dead_code)]

use futures::{SinkExt, StreamExt};
use iced::{
    Element, Subscription, Task, clipboard, time, widget::{Column, button, column, container, pick_list, row, scrollable, shader::wgpu::core::id, text, text_input}
};
use std::{sync::{Arc, Mutex}};
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};
use crate::api::{B, C, Lobby, S, ServerMessage};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerCount { P3, P4, P5, P6 }

impl PlayerCount {
    fn to_usize(self) -> usize {
        match self {
            PlayerCount::P3 => 3,
            PlayerCount::P4 => 4,
            PlayerCount::P5 => 5,
            PlayerCount::P6 => 6,
        }
    }
}

impl std::fmt::Display for PlayerCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerCount::P3 => write!(f, "3"),
            PlayerCount::P4 => write!(f, "4"),
            PlayerCount::P5 => write!(f, "5"),
            PlayerCount::P6 => write!(f, "6"),
        }
    }
}
struct App {
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    msg: String,
    ip: String,
    menu: MenuState,
    
    host_name: String,
    host_player_count: PlayerCount,
    
    join_name: String,
    
    lobby: Option<Lobby>,
    chat_input: String,
    server_messages: Vec<String>,
    last_msg: String,
}

#[derive(Debug, Clone)]
enum MenuState { Main, Host, Join, Rules, Lobby, Playing }

impl Default for App {
    fn default() -> Self {
        Self {
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            msg: String::new(),

            menu: MenuState::Main,
            
            host_name: "".to_string(),
            host_player_count: PlayerCount::P4,

            join_name: "".to_string(),

            lobby: Some(Lobby { players: Vec::new(), chat: Vec::new() }),
            chat_input: String::new(),
            server_messages: Vec::new(),
            ip: String::from("ws://localhost:3000"),
            last_msg: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Navigate(MenuState),

    Host,
    HostNameChanged(String),
    HostPlayerCountChanged(PlayerCount),
    JoinNameChanged(String),
    ServerAddressChanged(String),
    CopyToClipboard(String),

    SendChat,
    ChatInputChanged(String),

    CreateLobby,
    Connect,
    ToggleReady(u64),
    StartGame,

    GameRules,
    BackToMenu,
    CloseGame,
    
    Tick,
}

fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    match msg {
        AppMessage::Navigate (menu) => {
            state.menu = menu;
            Task::none()
        }
        AppMessage::Host => {
            // Host a server.
            if !state.connected {
                let _ = crate::server::start_server();
                std::thread::sleep(std::time::Duration::from_millis(300));
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, "127.0.0.1".into()));
                });
                state.connected = true;
                let local_ip = crate::server::local_ip();
                state.msg = format!("Hosting on {}", local_ip);
                state.ip = local_ip;
            }
            state.menu = MenuState::Host;
            Task::none()
        }
        AppMessage::HostNameChanged(name) => {
            state.host_name = name;
            Task::none()
        }
        AppMessage::HostPlayerCountChanged(count) => {
            state.host_player_count = count;
            Task::none()
        }
        AppMessage::JoinNameChanged(name) => {
            state.join_name = name;
            Task::none()
        }
        AppMessage::ServerAddressChanged(addr) => {
            state.ip = addr;
            Task::none()
        }
        AppMessage::CopyToClipboard(addr) => {
            state.last_msg = "Server address copied to clipboard.".to_string();
            clipboard::write(addr)
        }
        
        AppMessage::SendChat => {
            // Send chat message to server.
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::ChatMessage {sender: state.join_name.clone(), message: state.chat_input.clone() });
                    state.last_msg = "Sending chat message...".to_string();
                    state.chat_input.clear();
                }
            }
            Task::none()
        }
        AppMessage::ChatInputChanged(input) => {
            state.chat_input = input;
            Task::none()
        }
        AppMessage::CreateLobby => {
            // Send JoinLobby message to server.
            println!("Creating lobby...");
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby { name: state.host_name.clone() });
                    state.last_msg = "Creating lobby...".to_string();
                }
            }
            state.menu = MenuState::Lobby;
            Task::none()
        }
        AppMessage::Connect => {
            // Connect to "localhost:3000" without starting a server.
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                let ip = state.ip.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, ip));
                });
                state.connected = true;
                state.msg = "Connecting...".into();
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby { name: state.join_name.clone() });
                    state.last_msg = "Joining lobby...".to_string();
                }
            }
            state.menu = MenuState::Lobby;
            Task::none()
        }

        AppMessage::ToggleReady(p) => {
            // Find the player and toggle their ready state
            let new_ready_state = if let Some(lobby) = &state.lobby {
                if let Some(player) = lobby.players.iter().find(|pl| pl.id == p) {
                    !player.ready
                } else {
                    false
                }
            } else {
                false
            };
            
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetReady { ready: new_ready_state });
                    state.last_msg = format!("Set ready: {}", new_ready_state);
                }
            }
            Task::none()
        }
        AppMessage::StartGame => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::StartGame);
                    state.last_msg = "Starting game...".to_string();
                }
            }
            Task::none()
        }
        
        AppMessage::GameRules => {
            state.menu = MenuState::Rules;
            Task::none()
        }
        AppMessage::BackToMenu => {
            state.menu = MenuState::Main;
            Task::none()
        }
        AppMessage::CloseGame => {
            //close the Application
            std::process::exit(0);
        }
        
        AppMessage::Tick => {
            // Check for messages from the server.
            if let Ok(guard) = state.server_rx.lock() {
                if let Some(ref rx) = *guard {
                    while let Ok(msg) = rx.try_recv() {
                        println!("Received: {:?}", msg);
                        match msg {
                            ServerMessage::Server(s) => match s {
                                S::HandshakeConfirmation { version, supported } => {
                                    state.last_msg = format!("Handshake: version {version}, supported {supported}");
                                }
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
                                    state.last_msg = format!("Your turn: {} cards",
                                                             valid_cards.len());
                                }
                                S::InvalidMove { reason } => {
                                    state.last_msg = format!("Invalid move: {}", reason);
                                }
                                
                            }
                            ServerMessage::Broadcast(b) => match b {
                                B::LobbyState { lobby } => {
                                    state.last_msg = format!("Lobby: {} players", lobby.as_ref().unwrap().players.len());
                                    state.lobby = lobby;
                                }
                                B::ChatMessage { sender, message } => {
                                    state.last_msg = format!("Chat from {}: {}", sender, message);
                                    state.server_messages.push(format!("{}: {}", sender, message));
                                    if let Some(ref mut lobby) = state.lobby {
                                        lobby.chat.push((sender.to_string(), message));
                                    }
                                }
                                B::GameStarted { players } => {
                                    state.last_msg = format!("Game started: {} players",
                                                             players.len());
                                }
                                B::RoundStarted { round, cards_per_player,
                                                  trump } => {
                                    state.last_msg = format!("Round {} started: {} cards", round,
                                                             cards_per_player);
                                }
                                B::BiddingStarted { starting_player,
                                                    cards_per_player } => {
                                    state.last_msg = format!("Bidding started: {} cards",
                                                             cards_per_player);
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
                                B::RoundFinished { scores,
                                                   won_amounts } => {
                                    state.last_msg = format!("Round finished");
                                }
                                B::GameFinished { final_scores,
                                                  winner } => {
                                    state.last_msg = format!("Game won by {}", winner);
                                }
                            }
                        }
                    }
                }
            }
            Task::none()
        } // send C messages if needed
    }
}

async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver, ip: String) {
    let url = format!("ws://{}:3000/ws", ip);
    println!("Attempting to connect to {}...", url);
    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            println!("WebSocket started!");
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (srv_tx, srv_rx) = std::sync::mpsc::channel();

            *ws_tx.lock().unwrap() = Some(tx);
            *server_rx.lock().unwrap() = Some(srv_rx);
            println!("Receiver set successfully!");

            // Send a task.
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if write.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
            });

            // Recieve a task and directly parse it as a ServerMessage.
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

            // Wait for either task to complete.
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
    match state.menu {
        MenuState::Main => view_main_menu(),
        MenuState::Host => view_host_menu(state),
        MenuState::Join => view_join_menu(state),
        MenuState::Rules => view_rules_menu(),
        MenuState::Lobby => view_lobby_menu(state),
        MenuState::Playing => view_gameplay(state),
    }

}


fn subscription(state: &App) -> Subscription<AppMessage> {
    if state.connected {
        time::every(Duration::from_millis(100)).map(|_| AppMessage::Tick)
    } else {
        Subscription::none()
    }
}

fn view_main_menu<'a>() -> Element<'a, AppMessage> {
    let content = column![
        text("Wizard - Main Menu").size(40),
        button("Host").on_press(AppMessage::Host).padding(10),
        button("Join").on_press(AppMessage::Navigate(MenuState::Join)).padding(10),
        button("Gamerules").on_press(AppMessage::GameRules).padding(10),
        button("Exit Game").on_press(AppMessage::CloseGame).padding(10),
    ]
    .spacing(20)
    .align_x(iced::alignment::Horizontal::Center);

    container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

fn view_host_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let count_options = vec![PlayerCount::P3, PlayerCount::P4, PlayerCount::P5, PlayerCount::P6];
    let can_join = !state.host_name.is_empty();
    let content = column![
        text("Host").size(30),
        row![
            text(&state.ip),
            button("copy").on_press(AppMessage::CopyToClipboard(state.ip.clone())),
        ].spacing(10),
        text("Name:"),
        text_input("Your Name", &state.host_name).on_input(AppMessage::HostNameChanged),
        text("Player Count:"),
        pick_list(count_options.clone(), Some(state.host_player_count), |p| AppMessage::HostPlayerCountChanged(p)),
        button("Create Lobby").on_press_maybe(if can_join {
            Some(AppMessage::CreateLobby)
        } else {
            None
        }),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    container(content).center_x(iced::Fill).center_y(iced::Fill).into()
}

fn view_join_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let can_join = !state.ip.is_empty() && !state.join_name.is_empty();
    let content = column![
        text("Join").size(30),
        text("Name:"),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        button("Connect").on_press_maybe(if can_join {
            Some(AppMessage::Connect)
        } else {
            None
        }),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    container(content).center_x(iced::Fill).center_y(iced::Fill).into()
}

fn view_rules_menu<'a>() -> Element<'a, AppMessage> {
    let content = column![
        text("Game Rules").size(30),
        text("Here are the game rules (placeholder)."),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .align_x(iced::alignment::Horizontal::Left);

    container(content).center_x(iced::Fill).center_y(iced::Fill).into()
}

fn view_lobby_menu<'a>(state: &App) -> Element<'a, AppMessage> {
    if let Some(lobby) = &state.lobby {
        let mut player_rows = Column::new().spacing(10);
        for p in &lobby.players {
            let ready_text = if p.ready { "Bereit" } else { "Nicht bereit" };
            let toggle = button(ready_text).on_press(AppMessage::ToggleReady(p.id));
            let row = row![text(format!("{}{}", if p.is_host { "(Host) " } else { "" }, p.name)), toggle];
            player_rows = player_rows.push(row);
        }

        let mut chat_block = Column::new().spacing(5);
        for (sender, msg) in &lobby.chat {
            chat_block = chat_block.push(text(format!("{}: {}", sender, msg)));
        }

        // determine if start button should be enabled
        let can_start = lobby.players.len() >= 3 && lobby.players.iter().all(|p| p.ready);
        let start_button = 
        row![button("Starten").on_press_maybe(
                if can_start && lobby.players.iter().any(|p| p.is_host) { Some(AppMessage::StartGame)}
                else {
                    None
                }
            ),
            text(if !can_start {
                " (Warten auf Spieler...)"
                } 
                else if !lobby.players.iter().any(|p| p.is_host) {
                    " (Nur der Host kann starten)"
                } 
                else {
                    ""
                }
            )].spacing(5);

        let content = column![
            text("Lobby").size(30),
            row![
                text("Host Address:"),
                text_input("Address to share", &state.ip)
            ].spacing(10),
            text(format!("Spieler: {}/{}", lobby.players.len(), state.host_player_count.to_usize())),
            player_rows,
            scrollable(chat_block).height(150).width(400),
            row![
                text_input("Nachricht", &state.chat_input).on_input(AppMessage::ChatInputChanged),
                button("Senden").on_press(AppMessage::SendChat),
            ],
            start_button,
            button("Zurück zum Menü").on_press(AppMessage::BackToMenu)
        ]
        .spacing(10)
        .padding(20);

        container(content).center_x(iced::Fill).center_y(iced::Fill).into()
    } else {
        container(column![text("Keine Lobby vorhanden"), button("Zurück").on_press(AppMessage::BackToMenu)]).center_x(iced::Fill).center_y(iced::Fill).into()
    }

}
fn view_gameplay<'a>(state: &App) -> Element<'a, AppMessage> {
    let content = column![
        text("Gameplay Screen").size(30),
        text("Game in progress... (placeholder)"),
        button("Zurück zum Menü").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .align_x(iced::alignment::Horizontal::Left);

    container(content).center_x(iced::Fill).center_y(iced::Fill).into()
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
