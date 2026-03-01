pub mod audio;
mod update;
pub mod views;
mod ws;

use crate::api::{Card, Lobby, PlayerId, Suit};
use crate::client::audio::{Music, Sfx};
use crate::client::views::Button;
use crate::gameplay_ui::scoreboard::ScoreBoardInfo;
use crate::gameplay_ui::{GameStartInfo, GameView, GameViewMessage};
use crate::ui_element_traits::Message;
use iced::{Size, Subscription, Task, time, widget::image, window};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use strum::IntoEnumIterator;

pub use update::update;
pub use views::view;
pub use ws::{ServerMsgReceiver, WsConnection, connect_ws};

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
    Options,
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
    pub msg_queue: Vec<AppMessage>,
    pub msg_queue_delayed: Vec<AppMessage>,
    pub animation_count_down_latch: usize,

    pub connected: bool,
    pub connecting: bool,
    pub disconnected: bool,
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
    pub trump: Option<Card>,
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
    pub game_view: GameView,

    // UI Buttons (main menu)
    pub btn_host: crate::client::views::Button,
    pub btn_join: crate::client::views::Button,
    pub btn_rules: crate::client::views::Button,
    pub btn_options: crate::client::views::Button,
    pub btn_close: crate::client::views::Button,

    // Buttons for other menus
    pub btn_create_lobby: crate::client::views::Button,
    pub btn_back: crate::client::views::Button,
    pub btn_connect: crate::client::views::Button,
    pub btn_send_chat: crate::client::views::Button,
    pub btn_start_game: crate::client::views::Button,
    pub btn_back_to_menu: crate::client::views::Button,

    pub btn_ready_owned: crate::client::views::Button,

    // Audio
    pub audio: Option<crate::client::audio::Audio>,
    pub music_volume: i32,
    pub sfx_volume: i32,
    current_music: Option<Music>,
    pub img_main_menu: image::Handle,
    pub img_lobby_menu: image::Handle,
    pub img_background: image::Handle,
    pub img_menu_container: image::Handle,

    #[allow(dead_code)]
    pub card_images: HashMap<Card, image::Handle>,
}

impl App {
    pub fn scoreboard_info(&self) -> ScoreBoardInfo {
        ScoreBoardInfo {
            round_number: self.round_number,
            player_order: self.player_order.clone(),
            scores: self.scores.clone(),
            tricks_won: self.tricks_won.clone(),
            bids: self.bids.clone(),
            my_id: self.my_id,
            lobby: self.lobby.clone(),
            must_set_trump: self.must_set_trump,
            dealer: self.dealer,
            is_bidding_phase: self.is_bidding_phase,
            is_my_turn: self.is_my_turn,
            bid_input: self.bid_input.clone(),
            current_player: self.current_player,
        }
    }

    pub fn game_start_info(&self) -> GameStartInfo {
        GameStartInfo::new(self)
    }

    fn preload_card_images() -> HashMap<Card, image::Handle> {
        let mut map = HashMap::new();

        map.insert(
            Card::new(Suit::Red, crate::api::Value::Jester),
            image::Handle::from_path("assets/cards/variations/jester.png"),
        );
        map.insert(
            Card::new(Suit::Red, crate::api::Value::Wizard),
            image::Handle::from_path("assets/cards/variations/wizard.png"),
        );

        for suit in Suit::iter() {
            for num in 1..=13 {
                let card = Card::new(suit, crate::api::Value::Number(num));
                let path = card.img_path();
                map.insert(card, image::Handle::from_path(path));
            }
        }

        map
    }

    pub fn get_card_image(&self, card: Card) -> image::Handle {
        self.card_images.get(&card).cloned().unwrap_or_else(|| {
            //just in case
            image::Handle::from_path(card.img_path())
        })
    }
}

impl Default for App {
    fn default() -> Self {
        // Keep this value ins sync with the window size of the main function.
        let window_size: Size = Size::new(640.0, 480.0);
        let mut app = Self {
            window_size,
            msg_queue: Vec::new(),
            msg_queue_delayed: Vec::new(),
            animation_count_down_latch: 0,

            connected: false,
            connecting: false,
            disconnected: false,
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

            game_view: GameView::new(window_size),

            //Buttons
            btn_host: Button::new_host_button(0, 180, 44),
            btn_join: Button::new_join_button(1, 180, 44),
            btn_options: Button::new_options_button(4, 180, 44),
            btn_rules: Button::new_rules_button(2, 180, 44),
            btn_close: Button::new_close_button(3, 180, 44),

            btn_create_lobby: Button::new_create_lobby_button(10, 160, 40),
            btn_back: Button::new_back_button(11, 100, 36),
            btn_connect: Button::new_connect_button(12, 140, 40),
            btn_send_chat: Button::new_send_chat_button(13, 100, 36),
            btn_start_game: Button::new_start_game_button(14, 140, 40),
            btn_back_to_menu: Button::new_back_to_menu_button(15, 160, 40),

            btn_ready_owned: Button::new_ready_owned_button(20, 100, 36),

            img_main_menu: image::Handle::from_path("assets/wizard_main_menu.png"),
            img_lobby_menu: image::Handle::from_path("assets/wizard_lobby_menu.png"),
            img_background: image::Handle::from_path("assets/background_forall.png"),
            img_menu_container: image::Handle::from_path("assets/menu_container.png"),

            card_images: Self::preload_card_images(),
            audio: None,
            music_volume: 100,
            sfx_volume: 100,
            current_music: None,
        };

        if let Ok(mut a) = crate::client::audio::Audio::new() {
            let _ = a.load_clip("menu", "assets/audio/wizard_black_shores.mp3");
            let _ = a.load_clip("lobby", "assets/audio/wizard_clash_of_mages.mp3");
            let _ = a.load_clip("ingame", "assets/audio/wizard_peaceful.mp3");

            let _ = a.load_clip("click", "assets/audio/sfx_click.mp3");
            let _ = a.load_clip("game_over", "assets/audio/sfx_game_over.mp3");
            let _ = a.load_clip("card_hovered", "assets/audio/sfx_card_hovered.mp3");
            let _ = a.load_clip("card_shuffle", "assets/audio/sfx_card_shuffle.mp3");
            let _ = a.load_clip("card_dealed", "assets/audio/sfx_card_dealed.mp3");
            let _ = a.load_clip("card_played", "assets/audio/sfx_card_played.mp3");
            let _ = a.load_clip("card_error", "assets/audio/sfx_card_error.mp3");
            let _ = a.load_clip("shard_played", "assets/audio/sfx_shard_played.mp3");
            let _ = a.load_clip("mage_cast", "assets/audio/sfx_mage_cast.mp3");
            let _ = a.load_clip("witch_cast", "assets/audio/sfx_witch_cast.mp3");
            let _ = a.load_clip("elf_cast", "assets/audio/sfx_elf_cast.mp3");
            let _ = a.load_clip("knight_cast", "assets/audio/sfx_knight_cast.mp3");

            a.play_music(crate::client::audio::Music::Menu);
            app.audio = Some(a);
        }
        app
    }
}

impl App {
    pub fn set_menu(&mut self, menu: MenuState) {
        // Determine what music should play for this menu
        let target_music = match menu {
            MenuState::Main
            | MenuState::Host
            | MenuState::Join
            | MenuState::Rules
            | MenuState::Options => Some(Music::Menu),
            MenuState::Lobby => Some(Music::Lobby),
            MenuState::Playing | MenuState::PlayingTest => Some(Music::InGame),
        };

        // Only restart music if it's different from what's currently playing
        if target_music != self.current_music {
            if let Some(a) = &mut self.audio
                && let Some(music) = target_music
            {
                a.play_music(music);
            }
            self.current_music = target_music;
        }

        self.menu = menu;
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
    ToggleReady(PlayerId),
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

    // Gameview messages
    GameViewMessage(Box<GameViewMessage>),

    // Button messages from view widgets
    ButtonMessage(crate::client::views::ButtonMessage),

    // Audio messages
    MusicVolumeChanged(f32),
    SfxVolumeChanged(f32),
    // Animation Count Down Letch
    IncrementACDL(usize),
    DecrementACDL(usize),

    PlaySfx(Sfx),
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
    pub fn push_msg(&mut self, msg: impl Message) {
        self.push(msg.convert_msg_to_task())
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
