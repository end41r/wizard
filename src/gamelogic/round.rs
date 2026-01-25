// use iced::window::UserAttention;
use rand::rng;
use rand::Rng;
use std::collections::HashMap;

use strum::IntoEnumIterator;

type Err = &'static str;

use crate::gamelogic::card::{Card, Suit, Symbol};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    called: usize,
    hand: Vec<Card>,
    tricks_won: usize,
    pub points: i32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Round {
    pub players: HashMap<usize, Player>,
    pub order: Vec<usize>,
    pub round_number: usize,
    pub current_trick: Vec<(usize, Card)>,
    pub dealer: usize,
    pub current_player: usize,
    pub trump: Option<Suit>,
    pub dealer_needs_to_set_trump: bool,
    pub is_over: bool,
}

impl Round {
    pub fn new(round_number: usize, player_ids: &Vec<usize>) -> Self {
        if player_ids.is_empty() {
            panic!("Cannot create a round with no players");
        }
        let mut deck = vec![];
        for suit in Suit::iter() {
            for symbol in Symbol::iter() {
                deck.push(Card::new(suit, symbol));
            }
        }

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
        let dealer_position_in_player_ids = round_number % player_ids.len();
        let dealer_id = player_ids[dealer_position_in_player_ids];
        let current_player_position = (dealer_position_in_player_ids + 1) % player_ids.len();
        let current_player_id = player_ids[current_player_position];

        let mut order = vec![usize::max_value(); player_ids.len()];
        for i in 0..player_ids.len() {
            order[i] = player_ids[(current_player_position + i) % player_ids.len()];
        }

        let mut trump = None;

        let mut dealer_needs_to_set_trump = false;
        if !deck.is_empty() {
            let trump_card = draw_random_card(&mut deck);
            if trump_card.symbol != Symbol::Jester && trump_card.symbol != Symbol::Wizard {
                trump = Some(trump_card.suit);
            }
            dealer_needs_to_set_trump = trump_card.symbol == Symbol::Wizard;
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
        }
    }

    pub fn set_called(&mut self, player_id: usize, value: usize) -> Result<(), Err> {
        let player = match self.players.get_mut(&player_id) {
            Some(p) => p,
            None => return Err("Invalid Player ID."),
        };
        if player.called != usize::MAX {
            return Err("Player has already called");
        }
        player.called = value;
        Ok(())
    }

    pub fn set_trump(&mut self, player_id: usize, suit: Suit) -> Result<(), Err> {
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
    pub fn play_card(&mut self, player_id: usize, card: Card) -> Result<(), Err> {
        let player = self.players.get_mut(&player_id).unwrap();
        if self.current_player != player_id {
            return Err("It's not this player's turn");
        }
        // Check if the player has the card and remove it from their hand
        let card_position_result = player.hand.iter().position(|x| *x == card);
        match card_position_result {
            Some(index) => {
                player.hand.remove(index);
                self.current_trick.push((player_id, card));
            }
            None => {
                return Err("Player does not have this card in hand");
            }
        }
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
            panic!("Cannot calculate trick winner before all players have played a card. This should never happen");
        }

        let mut winner = self.current_trick[0];
        for (id, card) in self.current_trick.clone() {
            if card.symbol == Symbol::Wizard {
                winner = (id, card);
                break;
            }
            if self.better_than_winner(&winner.1, &card) {
                winner = (id, card);
            }
        }
        self.players.get_mut(&winner.0).unwrap().tricks_won += 1;

        if self.players[&0].hand.is_empty() {
            self.get_points();
            self.is_over = true;
        }

        self.current_trick.clear();
    }

    fn better_than_winner(&self, winner: &Card, card: &Card) -> bool {
        self.card_value(card) > self.card_value(winner)
    }

    fn card_value(&self, card: &Card) -> usize {
        let base: usize = match card.symbol {
            Symbol::Jester => 1,
            Symbol::Two => 2,
            Symbol::Three => 3,
            Symbol::Four => 4,
            Symbol::Five => 5,
            Symbol::Six => 6,
            Symbol::Seven => 7,
            Symbol::Eight => 8,
            Symbol::Nine => 9,
            Symbol::Ten => 10,
            Symbol::Jack => 11,
            Symbol::Queen => 12,
            Symbol::King => 13,
            Symbol::Ace => 14,
            Symbol::Wizard => 15,
        };

        let suit_bonus: usize = match self.trump {
            Some(trump_suit) if card.suit == trump_suit => 16,
            _ => 0,
        };

        base + suit_bonus
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

    pub fn to_state(&self) -> crate::gamelogic::game_state::RoundState {
        crate::gamelogic::game_state::RoundState {
            round_number: self.round_number,
            player_states: self
                .players
                .iter()
                .map(|(id, player)| {
                    let player_state = crate::gamelogic::game_state::PlayerState {
                        hand: player.hand.clone(),
                        called: player.called,
                        tricks_won: player.tricks_won,
                        id: *id,
                        points: player.points,
                    };
                    (*id, player_state)
                })
                .collect(),
            dealer: self.dealer,
            order: self.order.clone(),
            current_trick: self.current_trick.clone(),
            trump: self.trump,
            current_player: self.current_player,
            dealer_needs_to_set_trump: self.dealer_needs_to_set_trump,
        }
    }
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
