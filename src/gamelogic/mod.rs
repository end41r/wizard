pub mod game;
pub mod round;

use crate::api::{Card, PlayerId, Suit};
use std::collections::HashMap;

/// Events emitted by game logic - server translates these to messages
#[derive(Debug, Clone)]
pub enum GameEvent {
    // Game lifecycle
    GameStarted {
        players: Vec<PlayerId>,
    },
    GameFinished {
        final_scores: HashMap<PlayerId, i32>,
        winner: PlayerId,
    },

    // Round lifecycle
    RoundStarted {
        round: usize,
        cards_per_player: usize,
        trump: Option<Suit>,
    },
    RoundFinished {
        scores: HashMap<PlayerId, i32>,
        tricks_won: HashMap<PlayerId, usize>,
    },

    // Hands
    HandDealt {
        player: PlayerId,
        cards: Vec<Card>,
    },

    // Trump setting (when Wizard is drawn)
    DealerMustSetTrump {
        dealer: PlayerId,
    },
    TrumpSet {
        suit: Suit,
        by_dealer: PlayerId,
    },

    // Bidding
    BiddingStarted {
        starting_player: PlayerId,
        cards_per_player: usize,
    },
    BidRequest {
        player: PlayerId,
        min: usize,
        max: usize,
    },
    BidMade {
        player: PlayerId,
        amount: usize,
    },
    BiddingFinished {
        bids: HashMap<PlayerId, usize>,
    },

    // Trick play
    TrickStarted {
        leader: PlayerId,
    },
    TurnRequest {
        player: PlayerId,
        valid_cards: Vec<Card>,
    },
    CardPlayed {
        player: PlayerId,
        card: Card,
    },
    TrickFinished {
        winner: PlayerId,
        cards: Vec<(PlayerId, Card)>,
    },
}
