mod update;
mod views;
mod ws;
mod audio;

use crate::api::{Card, Lobby, PlayerId, Suit};
use crate::gameplay_ui::hand::{HandMessage, ViewableHand};
use iced::{time, window, Size, Subscription};
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

const TITLE_FONT: &[u8] = include_bytes!("../../assets/menu/MagicSchoolOne.ttf");

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
    Options,
    #[allow(dead_code)]
    PlayingTest,
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

    // UI Buttons (main menu)
    pub btn_host: crate::client::views::Button,
    pub btn_join: crate::client::views::Button,
    pub btn_rules: crate::client::views::Button,
    pub btn_exit: crate::client::views::Button,
    pub btn_options: crate::client::views::Button,

    // Buttons for other menus
    pub btn_create_lobby: crate::client::views::Button,
    pub btn_back: crate::client::views::Button,
    pub btn_connect: crate::client::views::Button,
    pub btn_send_chat: crate::client::views::Button,
    pub btn_start_game: crate::client::views::Button,
    pub btn_back_to_menu: crate::client::views::Button,

    pub btn_ready_owned: crate::client::views::Button,

    // Audio subsystem (may be None if audio initialization fails)
    pub audio: Option<crate::client::audio::Audio>,
    pub music_volume: i32,
    // prepared flag for a future mute button
    pub music_muted: bool,
}


impl Default for App {
    fn default() -> Self {
        // Keep this value ins sync with the window size of the main function.
        let window_size: Size = Size::new(640.0, 480.0);
        // build the base struct first so we can run fallible audio init
        let mut app = Self {
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

            //Buttons
            btn_host: crate::client::views::Button::new(0, "Host", "assets/menu/button1.png", 180, 44),
            btn_join: crate::client::views::Button::new(
                1,
                "Beitreten",
                "assets/menu/button1.png",
                180,
                44,
            ),
            btn_options: crate::client::views::Button::new(
                4,
                "Optionen",
                "assets/menu/button1.png",
                180,
                44,
            ),
            btn_rules: crate::client::views::Button::new(
                2,
                "Spielregeln",
                "assets/menu/button1.png",
                180,
                44,
            ),
            btn_exit: crate::client::views::Button::new(
                3,
                "Spiel verlassen",
                "assets/menu/button1.png",
                180,
                44,
            ),

            btn_create_lobby: crate::client::views::Button::new(
                10,
                "Lobby erstellen",
                "assets/menu/button1.png",
                160,
                40,
            ),
            btn_back: crate::client::views::Button::new(
                11,
                "zurück",
                "assets/menu/button1.png",
                100,
                36,
            ),
            btn_connect: crate::client::views::Button::new(
                12,
                "Verbinden",
                "assets/menu/button1.png",
                140,
                40,
            ),
            btn_send_chat: crate::client::views::Button::new(
                13,
                "Senden",
                "assets/menu/button1.png",
                100,
                36,
            ),
            btn_start_game: crate::client::views::Button::new(
                14,
                "Starten",
                "assets/menu/button1.png",
                140,
                40,
            ),
            btn_back_to_menu: crate::client::views::Button::new(
                15,
                "Zurück zum Menü",
                "assets/menu/button1.png",
                160,
                40,
            ),

            btn_ready_owned: crate::client::views::Button::new(
                20,
                "Bereit",
                "assets/menu/button1.png",
                100,
                36,
            ),

            audio: None,
            music_volume: 100,
            music_muted: false,
        };

        if let Ok(mut a) = crate::client::audio::Audio::new() {
            let _ = a.load_clip("menu", "assets/audio/wizard_black_shores.mp3");
            let _ = a.load_clip("lobby", "assets/audio/wizard_clash_of_mages.mp3");
            let _ = a.load_clip("ingame", "assets/audio/wizard_clash_of_mages.mp3");

            let _ = a.load_clip("click", "assets/audio/click.mp3");
            let _ = a.load_clip("card_place", "assets/audio/card_place.mp3");
            let _ = a.load_clip("win", "assets/audio/win.mp3");
            let _ = a.load_clip("lose", "assets/audio/lose.mp3");

            a.play_music(crate::client::audio::Music::Menu);
            app.audio = Some(a);
        }

        app
    }
}

impl App {
    /// Set the current menu and make sure the correct music plays for that screen.
    pub fn set_menu(&mut self, menu: MenuState) {
        // decide music first (avoids moving `menu` before we use it)
        if let Some(a) = &mut self.audio {
            match menu {
                MenuState::Main | MenuState::Host | MenuState::Join | MenuState::Rules | MenuState::Options => {
                    a.play_music(crate::client::audio::Music::Menu);
                }
                MenuState::Lobby => {
                    a.play_music(crate::client::audio::Music::Lobby);
                }
                MenuState::Playing | MenuState::PlayingTest => {
                    a.play_music(crate::client::audio::Music::InGame);
                }
            }
            if self.music_muted {
                a.set_music_muted(true);
            }
        }
        self.menu = menu;
    }

    /// not implemented
    pub fn toggle_music(&mut self) {
        self.music_muted = !self.music_muted;
        if let Some(a) = &mut self.audio {
            a.set_music_muted(self.music_muted);
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    WindowResized(Size),

    Navigate(MenuState),
    /// not yet implemented!!!
    ToggleMusic,

    Host,
    HostNameChanged(String),
    HostPlayerCountChanged(PlayerCount),
    JoinNameChanged(String),
    ServerAddressChanged(String),

    SendChat,
    ChatInputChanged(String),

    CreateLobby,
    Connect,
    ToggleReady(u64),
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

    // Button messages from view widgets
    ButtonMessage(crate::client::views::ButtonMessage),

    // Audio messages
    MusicVolumeChanged(f32),
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
