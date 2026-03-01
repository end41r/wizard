#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub static CARD_BACK_PATH: &str = "assets/cards/back.png";
pub static BUTTON1_PATH: &str = "assets/button1.png";
pub static FRAME_PLAYABLE_PATH: &str = "assets/cards/frame_green.png";
pub static FRAME_PLAYABLE_FOCUSED_PATH: &str = "assets/cards/frame_yellow.png";
pub static FALSE_PLAYED_PATH: &str = "assets/cards/false_played.png";

pub type PlayerId = u64;
pub type SessionId = u64;

pub trait TextColor {
    fn white(self) -> Self;
}

impl<'a> TextColor for iced::widget::Text<'a> {
    fn white(self) -> Self {
        self.color(iced::Color::WHITE)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Server(S),
    Broadcast(B),
    ConnectionClosed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum C {
    // Messages sent by the client.
    Handshake { version: usize },
    JoinLobby { name: String },
    LeaveLobby,
    ChatMessage { sender: String, message: String },
    SetReady { ready: bool },
    StartGame,
    SetPlayerCount { count: usize },

    Bid { amount: usize },
    SetTrump { suit: Suit },

    PlayCard { card: Card },

    RequestShutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum S {
    // Messages sent by the server.
    HandshakeConfirmation { version: usize, supported: bool },
    JoinConfirmation { ok: bool, id: PlayerId },
    Error { reason: String },

    HandDealt { cards: Vec<Card> },

    TrumpRequest,
    BidRequest { min: usize, max: usize },
    InvalidBid { reason: String },

    YourTurn { valid_cards: Vec<Card> },
    InvalidMove { reason: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum B {
    // Messages broadcasted by the server.
    LobbyState {
        lobby: Option<Lobby>,
    },
    ChatMessage {
        sender: String,
        message: String,
    },
    PlayerCountChanged {
        count: usize,
    },

    GameStarted {
        players: Vec<PlayerId>,
    },
    RoundStarted {
        round: usize,
        cards_per_player: usize,
        trump: Option<Card>,
    },
    DealerMustSetTrump {
        dealer: PlayerId,
    },
    TrumpSet {
        suit: Suit,
        by_dealer: PlayerId,
    },
    BiddingStarted {
        starting_player: PlayerId,
        cards_per_player: usize,
    },
    BidTurn {
        player: PlayerId,
    },
    BidMade {
        player: PlayerId,
        amount: usize,
    },
    BiddingFinished {
        bids: Vec<(PlayerId, usize)>,
    },

    PoolStarted {
        leader: PlayerId,
    },
    TurnChanged {
        player: PlayerId,
    },
    CardPlayed {
        player: PlayerId,
        card: Card,
    },

    PoolFinished {
        winner: PlayerId,
        cards: Vec<(PlayerId, Card)>,
    },

    RoundFinished {
        scores: Vec<(PlayerId, i32)>,
        won_amounts: Vec<(PlayerId, usize)>,
    },

    GameFinished {
        final_scores: Vec<(PlayerId, i32)>,
        winner: PlayerId,
    },
    /// Server is shutting down (host stopped server)
    ServerShutdown,
}

#[derive(Clone, Debug)]
pub struct Avatar {
    kind: AvatarKind,
    pose: AvatarPose,
    casting_finished: bool,
    continue_casting: bool,
}

impl Avatar {
    pub fn new(kind: AvatarKind) -> Self {
        Self {
            kind,
            pose: AvatarPose::Standing1,
            casting_finished: false,
            continue_casting: false,
        }
    }
    pub fn kind(&self) -> AvatarKind {
        self.kind
    }
    pub fn pose(&self) -> AvatarPose {
        self.pose
    }
    pub fn next_pose(&mut self) {
        match self.pose {
            AvatarPose::Standing1 => {
                if self.continue_casting {
                    self.pose = AvatarPose::Casting1
                } else {
                    self.pose = AvatarPose::Standing2
                }
            }
            AvatarPose::Standing2 => {
                if self.continue_casting {
                    self.pose = AvatarPose::Casting1
                } else {
                    self.pose = AvatarPose::Standing1
                }
            }
            AvatarPose::Casting1 => {
                if self.continue_casting && !self.casting_finished {
                    self.pose = AvatarPose::Casting2
                } else {
                    self.pose = AvatarPose::Standing1;
                    self.casting_finished = false;
                    self.continue_casting = false;
                }
            }
            AvatarPose::Casting2 => {
                self.pose = AvatarPose::Casting1;
                self.casting_finished = true;
            }
        }
    }
    pub fn start_casting(&mut self) {
        self.continue_casting = true;
    }
    pub fn is_casting(&self) -> bool {
        self.continue_casting
    }
    pub fn img_path(&self) -> String {
        let mut path: String = "assets/avatars/".to_owned();
        match &self.kind {
            AvatarKind::Elf => {
                path.push_str("elf/elf_");
            }
            AvatarKind::Knight => {
                path.push_str("knight/knight_");
            }
            AvatarKind::Mage => {
                path.push_str("mage/mage_");
            }
            AvatarKind::Witch => {
                path.push_str("witch/witch_");
            }
        }
        match &self.pose {
            AvatarPose::Standing1 => {
                path.push_str("standing1");
            }
            AvatarPose::Standing2 => {
                path.push_str("standing2");
            }
            AvatarPose::Casting1 => {
                path.push_str("casting1");
            }
            AvatarPose::Casting2 => {
                path.push_str("casting2");
            }
        }
        path.push_str(".png");
        path
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AvatarKind {
    Elf,
    Knight,
    Mage,
    Witch,
}

impl AvatarKind {
    pub fn as_avatar(&self) -> Avatar {
        Avatar::new(*self)
    }
    pub fn img_path(&self, pose: AvatarPose) -> String {
        let mut path: String = "assets/avatars/".to_owned();
        match &self {
            AvatarKind::Elf => {
                path.push_str("elf/elf_");
            }
            AvatarKind::Knight => {
                path.push_str("knight/knight_");
            }
            AvatarKind::Mage => {
                path.push_str("mage/mage_");
            }
            AvatarKind::Witch => {
                path.push_str("witch/witch_");
            }
        }
        match pose {
            AvatarPose::Standing1 => {
                path.push_str("standing1");
            }
            AvatarPose::Standing2 => {
                path.push_str("standing2");
            }
            AvatarPose::Casting1 => {
                path.push_str("casting1");
            }
            AvatarPose::Casting2 => {
                path.push_str("casting2");
            }
        }
        path.push_str(".png");
        path
    }
    pub fn shard_path(&self) -> String {
        let mut path: String = "assets/avatars/".to_owned();
        match &self {
            AvatarKind::Elf => {
                path.push_str("elf/");
            }
            AvatarKind::Knight => {
                path.push_str("knight/");
            }
            AvatarKind::Mage => {
                path.push_str("mage/");
            }
            AvatarKind::Witch => {
                path.push_str("witch/");
            }
        }
        path.push_str("shard.png");
        path
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvatarPose {
    Standing1,
    Standing2,
    Casting1,
    Casting2,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

impl Card {
    pub fn new(suit: Suit, value: Value) -> Self {
        Self { suit, value }
    }

    pub fn img_path(&self) -> String {
        let mut path: String = "assets/cards/variations/".to_owned();
        if self.value == Value::Jester {
            path.push_str("jester");
        } else if self.value == Value::Wizard {
            path.push_str("wizard");
        } else {
            match self.suit {
                Suit::Blue => {
                    path.push_str("blue_");
                }
                Suit::Green => {
                    path.push_str("green_");
                }
                Suit::Red => {
                    path.push_str("red_");
                }
                Suit::Yellow => {
                    path.push_str("yellow_");
                }
            }
            if let Value::Number(number) = self.value {
                path.push_str(number.to_string().as_str());
            }
        }
        path.push_str(".png");
        path
    }

    pub fn glow_path(&self) -> String {
        if self.value == Value::Jester {
            // glow card will treat this as no existant glow yet
            "".to_string()
        } else if self.value == Value::Wizard {
            // glow card will treat this as a existant glow bu won't find the image,
            // so glow is invisible
            "NOT VALID".to_string()
        } else {
            let mut path: String = "assets/cards/".to_owned();
            match self.suit {
                Suit::Blue => {
                    path.push_str("glow_blue");
                }
                Suit::Green => {
                    path.push_str("glow_green");
                }
                Suit::Red => {
                    path.push_str("glow_red");
                }
                Suit::Yellow => {
                    path.push_str("glow_yellow");
                }
            }
            path.push_str(".png");
            path
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash, EnumIter)]
pub enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash, EnumIter)]
pub enum Value {
    Jester,
    Number(u8), // 1-13
    Wizard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub avatar: AvatarKind,
    pub ready: bool,
    pub is_host: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lobby {
    pub players: Vec<Player>,
    /// Contains tuples of (sender, message).
    pub chat: Vec<(String, String)>,
}
