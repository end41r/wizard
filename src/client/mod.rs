mod update;
mod views;
mod ws;

use crate::api::{Card, Lobby, PlayerId, Suit};
use crate::client::views::Button;
use crate::gameplay_ui::hand::{HandMessage, ViewableHand};
use crate::gameplay_ui::table::{
    TableMessage, ViewableTable,
};
use crate::ui_element_traits::Message;
use iced::{time, window, Size, Subscription, Task};
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

const TITLE_FONT: &[u8] = include_bytes!("../../assets/MagicSchoolOne.ttf");

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
    #[allow(dead_code)]
    PlayingTest,
}

impl Message for MenuState {
    fn convert_msg_from(msg: Self) -> AppMessage {
        AppMessage::Navigate(msg)
    }
}

pub struct App {
    window_size: Size,

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
    pub hand: Vec<Card>,
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

    // Gameplay view state
    pub viewable_hand: ViewableHand,
    pub viewable_table: ViewableTable,

    // UI Buttons (main menu)
    pub btn_host: crate::client::views::Button,
    pub btn_join: crate::client::views::Button,
    pub btn_rules: crate::client::views::Button,
    pub btn_close: crate::client::views::Button,

    // Buttons for other menus
    pub btn_create_lobby: crate::client::views::Button,
    pub btn_back: crate::client::views::Button,
    pub btn_connect: crate::client::views::Button,
    pub btn_send_chat: crate::client::views::Button,
    pub btn_start_game: crate::client::views::Button,
    pub btn_back_to_menu: crate::client::views::Button,

    pub btn_ready_owned: crate::client::views::Button,
}

impl Default for App {
    fn default() -> Self {
        // Keep this value ins sync with the window size of the main function.
        let window_size: Size = Size::new(640.0, 480.0);
        Self {
            window_size,

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
            hand: Vec::new(),
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

            viewable_hand: ViewableHand::new(window_size),
            viewable_table: ViewableTable::new(window_size),

            //Buttons
            btn_host: Button::new_host_button(0, 180, 44),
            btn_join: Button::new_join_button(1, 180, 44),
            btn_rules: Button::new_rules_button(2, 180, 44),
            btn_close: Button::new_close_button(3, 180, 44),

            btn_create_lobby: Button::new_create_lobby_button(10, 160, 40),
            btn_back: Button::new_back_button(11, 100, 36),
            btn_connect: Button::new_connect_button(12, 140, 40),
            btn_send_chat: Button::new_send_chat_button(13, 100, 36),
            btn_start_game: Button::new_start_game_button(14, 140, 40),
            btn_back_to_menu: Button::new_back_to_menu_button(15, 160, 40),

            btn_ready_owned: Button::new_ready_owned_button(20, 100, 36),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    WindowResized(Size),

    Navigate(MenuState),

    Host,
    HostNameChanged(String),
    HostPlayerCountChanged(PlayerCount),
    JoinNameChanged(String),
    ServerAddressChanged(String),

    SendChat,
    ChatInputChanged(String),

    CreateLobby,
    Connect,
    ToggleReady(usize),
    StartGame,

    ServerTick,
    AnimationTick,

    // Gameplay messages
    BidInputChanged(String),
    SubmitBid,
    PlayCard(Card),
    SetTrump(Suit),

    GameRules,
    BackToMenu,
    CloseGame,

    // Gameplay view messages
    HandMessage(HandMessage),
    TableMessage(TableMessage),

    // Button messages from view widgets
    ButtonMessage(crate::client::views::ButtonMessage),
}

impl Message for AppMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        msg
    }
}

pub struct TaskBatcher {
    tasks: Vec<Task<AppMessage>>,
}

impl TaskBatcher {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    pub fn push(&mut self, task: Task<AppMessage>) {
        if task.units() != 0 {
            self.tasks.push(task)
        }
    }
    pub fn push_mult<const SIZE: usize>(&mut self, tasks: [Task<AppMessage>; SIZE]) {
        self.tasks
            .extend(tasks.into_iter().filter(|task| task.units() != 0));
    }
    pub fn batch(self) -> Task<AppMessage> {
        Task::batch(self.tasks)
    }
    // AI-Usage: Gemini for learning how to put an array into a function and filter it.
    pub fn instant_batch<const SIZE: usize>(tasks: [Task<AppMessage>; SIZE]) -> Task<AppMessage> {
        Task::batch(tasks.into_iter().filter(|task| task.units() != 0))
    }
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    let mut subscriptions: Vec<Subscription<AppMessage>> = vec![];
    subscriptions.push(window::resize_events().map(|(_, size)| AppMessage::WindowResized(size)));
    subscriptions.push(time::every(Duration::from_millis(16)).map(|_| AppMessage::AnimationTick));
    if state.connected || state.connecting {
        subscriptions.push(time::every(Duration::from_millis(100)).map(|_| AppMessage::ServerTick));
    }
    Subscription::batch(subscriptions)
}

pub fn main() -> iced::Result {
    use std::borrow::Cow;
    iced::application(App::default, update, view)
        .title("Wizard")
        .subscription(subscription)
        // Keep this value in sync with the App::default function.
        .window_size(Size::new(300.0, 800.0))
        .settings(iced::Settings {
            fonts: vec![Cow::Borrowed(TITLE_FONT)],
            ..Default::default()
        })
        .run()
}
