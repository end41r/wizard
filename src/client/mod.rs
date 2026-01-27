mod update;
mod views;
mod ws;

use crate::api::{Card, Lobby, PlayerId, Suit};
use iced::{time, Subscription};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use update::update;
pub use views::view;
pub use ws::{connect_ws, ServerMsgReceiver, WsConnection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCount {
    P3,
    P4,
    P5,
    P6,
}

impl PlayerCount {
    pub fn to_usize(self) -> usize {
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

#[derive(Debug, Clone)]
pub enum MenuState {
    Main,
    Host,
    Join,
    Rules,
    Lobby,
    Playing,
}

pub struct App {
    pub connected: bool,
    pub connecting: bool,
    pub ws_tx: WsConnection,
    pub server_rx: ServerMsgReceiver,
    pub msg: String,
    pub ip: String,
    pub menu: MenuState,

    pub host_name: String,
    pub host_player_count: PlayerCount,

    pub join_name: String,
    pub my_id: Option<PlayerId>,

    pub lobby: Option<Lobby>,
    pub chat_input: String,
    pub server_messages: Vec<String>,
    pub last_msg: String,

    // Gameplay state
    pub game_log: Vec<String>,
    pub my_hand: Vec<Card>,
    pub current_trick: Vec<(PlayerId, Card)>,
    pub trump: Option<Suit>,
    pub round_number: usize,
    pub is_my_turn: bool,
    pub is_bidding_phase: bool,
    pub must_set_trump: bool,
    pub current_player: Option<PlayerId>,
    pub player_order: Vec<PlayerId>,
    pub bids: HashMap<PlayerId, usize>,
    pub tricks_won: HashMap<PlayerId, usize>,
    pub scores: HashMap<PlayerId, i32>,
    pub bid_input: String,
    pub valid_cards: Vec<Card>,
    pub dealer: Option<PlayerId>,
    pub game_over: bool,
    pub winner: Option<PlayerId>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            connected: false,
            connecting: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            msg: String::new(),

            menu: MenuState::Main,

            host_name: "".to_string(),
            host_player_count: PlayerCount::P4,
            join_name: "".to_string(),

            my_id: None,

            lobby: Some(Lobby {
                players: Vec::new(),
                chat: Vec::new(),
            }),
            chat_input: String::new(),
            server_messages: Vec::new(),
            ip: String::from("localhost"),
            last_msg: String::new(),

            // Gameplay defaults
            game_log: Vec::new(),
            my_hand: Vec::new(),
            current_trick: Vec::new(),
            trump: None,
            round_number: 0,
            is_my_turn: false,
            is_bidding_phase: false,
            must_set_trump: false,
            current_player: None,
            player_order: Vec::new(),
            bids: HashMap::new(),
            tricks_won: HashMap::new(),
            scores: HashMap::new(),
            bid_input: String::new(),
            valid_cards: Vec::new(),
            dealer: None,
            game_over: false,
            winner: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
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

    // Gameplay messages
    BidInputChanged(String),
    SubmitBid,
    PlayCard(Card),
    SetTrump(Suit),

    GameRules,
    BackToMenu,
    CloseGame,

    Tick,
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    if state.connected || state.connecting {
        time::every(Duration::from_millis(100)).map(|_| AppMessage::Tick)
    } else {
        Subscription::none()
    }
}

pub fn main() -> iced::Result {
    iced::application("Wizard", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(640.0, 480.0),
            resizable: true,
            ..Default::default()
        })
        .run()
}
