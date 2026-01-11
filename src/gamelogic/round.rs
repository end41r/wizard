use rand::Rng;
use rand::rng;
use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::gamelogic::card;
use crate::gamelogic::card::{Card, Suit, Symbol};
use crate::gamelogic::round;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    called: usize,
    hand: Vec<Card>,
    tricks_won: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Round {
    pub players: HashMap<usize,Player>,
    pub order: Vec<usize>,
    pub round_number: usize,
    pub current_trick: Vec<(usize, Card)>,
    pub dealer: usize,
    pub current_player: usize,
    pub trump: Option<Suit>,
    pub prediction: usize,
    pub dealer_needs_to_set_trump: bool,
}

impl Round {
    pub fn new(round_number: usize, player_ids: &Vec<usize>) -> Self {

        if player_ids.len() == 0 {
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
            players.insert(id.clone(), Player { called: usize::MAX, hand: draw_random_cards(&mut deck, round_number + 1), tricks_won: 0 });
        }
        let dealer_position_in_player_ids = round_number % player_ids.len();
        let dealer_id = player_ids[dealer_position_in_player_ids];
        let current_player_position = (dealer_position_in_player_ids + 1) % player_ids.len();
        let current_player_id = player_ids[current_player_position];

        let mut order = vec![0; player_ids.len()];
        for i in 0..player_ids.len()-1 {
            order[i] = player_ids[(current_player_position + i) % player_ids.len()];
        }

        let mut trump = None;

        let mut dealer_needs_to_set_trump = false;
        if deck.len() > 0 {
            let trump_card = draw_random_card(&mut deck);
            if trump_card.symbol != Symbol::Jester && trump_card.symbol != Symbol::Wizard {
                trump = Some(trump_card.suit);
            }
            dealer_needs_to_set_trump = trump_card.symbol == Symbol::Wizard;
        }

        Round {
            players: players,
            order: order,
            current_trick: vec![],
            dealer: dealer_id,
            current_player: current_player_id,
            trump: trump,
            dealer_needs_to_set_trump: dealer_needs_to_set_trump,
            prediction: 0,
            round_number: round_number    
        }
    }

    pub fn to_state(&self) -> crate::gamelogic::game_state::RoundState {
        crate::gamelogic::game_state::RoundState {
            round_number: self.round_number,
            player_states: self.players.iter().map(|(id, player)| {
                let player_state = crate::gamelogic::game_state::PlayerState {
                    hand: player.hand.clone(),
                    called: player.called,
                    tricks_won: player.tricks_won,
                };
                (id.clone(), player_state)
            }).collect(),
            dealer: self.dealer,
            trump: self.trump,
            current_player: self.current_player,
            dealer_needs_to_set_trump: self.dealer_needs_to_set_trump,
        }
    }

}

fn draw_random_cards(deck: &mut Vec<Card>, count: usize) -> Vec<Card> {
    let mut cards = vec![];

    for i in 0..count {
        cards.push(draw_random_card(deck));
    }

    cards
}

fn draw_random_card(deck: &mut Vec<Card>) -> Card {
    let mut rng: rand::prelude::ThreadRng = rng();
    let index = rng.random_range(0..deck.len());
    deck.remove(index)
}
