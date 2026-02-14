// use iced::window::UserAttention;
use rand::rng;
use rand::Rng;
use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::api::PlayerId;

use crate::api::{Card, Suit, Value};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub called: usize,
    pub hand: Vec<Card>,
    pub tricks_won: usize,
    pub points: i32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Round {
    pub players: HashMap<PlayerId, Player>,
    pub order: Vec<PlayerId>,
    pub round_number: usize,
    pub current_trick: Vec<(PlayerId, Card)>,
    pub dealer: PlayerId,
    pub current_player: PlayerId,
    pub trump: Option<Suit>,
    pub dealer_needs_to_set_trump: bool,
    pub is_over: bool,
    pub bidding_phase: bool,
    pub current_bidder_index: usize,
}

impl Round {
    pub fn new(round_number: usize, player_ids: &Vec<PlayerId>) -> Self {
        if player_ids.is_empty() {
            panic!("Cannot create a round with no players");
        }
        let mut deck: Vec<Card> = build_deck();

        let mut players = HashMap::new();
        for id in player_ids {
            players.insert(
                *id,
                Player {
                    called: usize::MAX,
                    hand: draw_random_cards(&mut deck, round_number + 1),
                    tricks_won: 0,
                    points: 0,
                },
            );
        }

        let dealer_position = round_number % player_ids.len();
        let dealer_id = player_ids[dealer_position];
        let current_player_position = (dealer_position + 1) % player_ids.len();
        let current_player_id = player_ids[current_player_position];

        let mut order = vec![PlayerId::MAX; player_ids.len()];

        for i in 0..player_ids.len() {
            order[i] = player_ids[(current_player_position + i) % player_ids.len()];
        }

        let mut trump = None;

        let mut dealer_needs_to_set_trump = false;
        if !deck.is_empty() {
            let trump_card = draw_random_card(&mut deck);
            if trump_card.value != Value::Jester && trump_card.value != Value::Wizard {
                trump = Some(trump_card.suit);
            }
            dealer_needs_to_set_trump = trump_card.value == Value::Wizard;
        }

        Round {
            players,
            order,
            current_trick: vec![],
            dealer: dealer_id,
            current_player: current_player_id,
            trump,
            dealer_needs_to_set_trump,
            round_number,
            is_over: false,
            bidding_phase: true,
            current_bidder_index: 0,
        }
    }

    pub fn set_called(&mut self, player_id: PlayerId, value: usize) -> Result<(), &'static str> {
        if !self.bidding_phase {
            return Err("Bidding phase is over");
        }
        if self.dealer_needs_to_set_trump {
            return Err("Dealer must set trump first");
        }
        let expected_bidder = self.order[self.current_bidder_index];
        if player_id != expected_bidder {
            return Err("Wrong player's turn to bid");
        }
        let player = match self.players.get_mut(&player_id) {
            Some(p) => p,
            None => return Err("Invalid Player ID."),
        };
        if player.called != usize::MAX {
            return Err("Player has already called");
        }

        // Validate bid value (0 to number of cards in hand)
        let max_bid = self.round_number + 1;
        if value > max_bid {
            return Err("Bid exceeds maximum allowed");
        }

        player.called = value;

        // No use of a % wrap-around because we wouldnt know bidding is over
        self.current_bidder_index += 1;
        if self.current_bidder_index >= self.order.len() {
            self.bidding_phase = false;
            self.current_bidder_index = 0;
        }

        Ok(())
    }

    pub fn set_trump(&mut self, player_id: PlayerId, suit: Suit) -> Result<(), &'static str> {
        if self.dealer != player_id {
            return Err("Only the dealer can set the trump");
        }
        if !self.dealer_needs_to_set_trump {
            return Err("Trump has already been set for this round");
        }
        self.trump = Some(suit);
        self.dealer_needs_to_set_trump = false;
        Ok(())
    }

    pub fn play_card(&mut self, player_id: PlayerId, card: Card) -> Result<(), &'static str> {
        if self.bidding_phase {
            return Err("Cannot play cards during bidding phase");
        }
        if self.dealer_needs_to_set_trump {
            return Err("Dealer must set trump first");
        }
        if self.current_player != player_id {
            return Err("Wrong player's turn to play");
        }

        let player = self.players.get(&player_id).unwrap();

        if !player.hand.contains(&card) {
            return Err("Player does not have this card in hand");
        }

        if card.value != Value::Wizard
            && card.value != Value::Jester
            && !self.current_trick.is_empty()
        {
            let lead_suit = self
                .current_trick
                .iter()
                .find(|(_, c)| c.value != Value::Jester && c.value != Value::Wizard)
                .map(|(_, c)| c.suit);

            if let Some(lead) = lead_suit {
                let has_lead_suit = player.hand.iter().any(|c| {
                    c.suit == lead && c.value != Value::Wizard && c.value != Value::Jester
                });
                if has_lead_suit && card.suit != lead {
                    return Err("Must be playing a suit of the first card played");
                }
            }
        }

        let player = self.players.get_mut(&player_id).unwrap();
        let card_position = player.hand.iter().position(|x| *x == card).unwrap();
        player.hand.remove(card_position);
        self.current_trick.push((player_id, card));
        let current_player_position = self.order.iter().position(|x| *x == player_id).unwrap();
        let next_player_position = (current_player_position + 1) % self.order.len();
        self.current_player = self.order[next_player_position];
        if self.order.len() == self.current_trick.len() {
            self.calc_trick_winner();
        }
        Ok(())
    }

    fn calc_trick_winner(&mut self) {
        if self.order.len() != self.current_trick.len() {
            panic!("Cannot calculate trick winner before all players have played a card.");
        }

        // Find the lead suit (first non-Jester, non-Wizard card)
        let lead_suit = self
            .current_trick
            .iter()
            .find(|(_, card)| card.value != Value::Jester && card.value != Value::Wizard)
            .map(|(_, card)| card.suit);

        let mut winner = self.current_trick[0];

        for (id, card) in self.current_trick.clone() {
            // First Wizard always wins
            if card.value == Value::Wizard {
                winner = (id, card);
                break;
            }
            if self.better_than_winner(&winner.1, &card, lead_suit) {
                winner = (id, card);
            }
        }

        self.players.get_mut(&winner.0).unwrap().tricks_won += 1;

        self.current_player = winner.0;

        let round_over = self
            .players
            .values()
            .next()
            .map(|p| p.hand.is_empty())
            .unwrap_or(true);
        if round_over {
            self.get_points();
            self.is_over = true;
        }

        self.current_trick.clear();
    }

    fn better_than_winner(&self, winner: &Card, card: &Card, lead_suit: Option<Suit>) -> bool {
        if card.value == Value::Jester {
            return false;
        }
        if winner.value == Value::Jester {
            return true;
        }
        if winner.value == Value::Wizard {
            return false;
        }

        self.card_value(card, lead_suit) > self.card_value(winner, lead_suit)
    }

    fn card_value(&self, card: &Card, lead_suit: Option<Suit>) -> usize {
        let base: usize = match card.value {
            Value::Jester => 0_usize,
            Value::Number(n) => n as usize,
            Value::Wizard => 100, // Wizard always highest
        };

        let is_trump = self.trump.map(|t| card.suit == t).unwrap_or(false);
        let is_lead = lead_suit.map(|l| card.suit == l).unwrap_or(false);

        // just to be sure :)
        if card.value == Value::Wizard {
            return usize::MAX;
        }
        if card.value == Value::Jester {
            return usize::MIN;
        }

        if is_trump {
            base + 32 // Trump cards beat everything except Wizard
        } else if is_lead {
            base + 16 // Lead suit beats off-suit
        } else {
            0
        }
    }

    fn get_points(&mut self) {
        for player in self.players.values_mut() {
            let called = player.called as i32;
            let won = player.tricks_won as i32;

            let points = if called == won {
                20 + 10 * won
            } else {
                -10 * (called - won).abs()
            };
            player.points += points;
        }
    }
}

fn build_deck() -> Vec<Card> {
    let mut deck = vec![];
    for suit in Suit::iter() {
        deck.push(Card::new(suit, Value::Jester));
        deck.push(Card::new(suit, Value::Wizard));
        for num in 1..=13 {
            deck.push(Card::new(suit, Value::Number(num)));
        }
    }
    deck
}

fn draw_random_cards(deck: &mut Vec<Card>, count: usize) -> Vec<Card> {
    let mut cards = vec![];

    for _i in 0..count {
        cards.push(draw_random_card(deck));
    }

    cards
}

fn draw_random_card(deck: &mut Vec<Card>) -> Card {
    let mut rng: rand::prelude::ThreadRng = rng();
    let index = rng.random_range(0..deck.len());
    deck.remove(index)
}

// A temporary function for getting a random card used for ViewableCard::build_test_cards.
pub fn random_card() -> Card {
    draw_random_card(build_deck().as_mut())
}
