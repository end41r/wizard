#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        bids: HashMap<PlayerId, usize>,
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
        scores: HashMap<PlayerId, usize>,
        won_amounts: HashMap<PlayerId, usize>,
    },

    GameFinished {
        final_scores: HashMap<PlayerId, usize>,
        winner: PlayerId,
    },
    /// Server is shutting down (host stopped server)
    ServerShutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Narre,
    Number(u8),
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
