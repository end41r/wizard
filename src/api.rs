#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub type PlayerId = u64;
pub type SessionId = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Server(S),
    Broadcast(B),
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
        trump: Option<Suit>,
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

impl Card {
    pub fn new(suit: Suit, value: Value) -> Self {
        Self { suit, value }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
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
    pub ready: bool,
    pub is_host: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lobby {
    pub players: Vec<Player>,
    /// Contains tuples of (sender, message).
    pub chat: Vec<(String, String)>,
}

pub fn get_card_path(card: Card) -> String {
    let mut path: String = "assets/cards/variations/".to_owned();
    if card.value == Value::Jester {
        path.push_str("jester");
    } else if card.value == Value::Wizard {
        path.push_str("wizard");
    } else {
        match card.suit {
            Suit::Blue => {path.push_str("diamond ");}
            Suit::Green => {path.push_str("club ");}
            Suit::Red => {path.push_str("heart ");}
            Suit::Yellow => {path.push_str("spade ");}
        }
        match card.value {
            Value::Number(number) => {
                path.push_str(number.to_string().as_str());
            }
            _ => ()
        }
    }
    path.push_str(".png");
    path
}
