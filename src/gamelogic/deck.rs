use strum::IntoEnumIterator;
use std::fmt::Display;

use crate::gamelogic::card::{Card, Suit, Symbol};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck = Self { cards: vec![] };
        for suit in Suit::iter() {
            for symbol in Symbol::iter() {
                deck.cards.push(Card::new(suit, symbol));
            }
        }
        deck
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cards)
    }
}

#[test]
fn deck_has_60_cards() {
    let deck = Deck::new();
    // println!("{}", deck)
    assert_eq!(deck.cards.len(), 60)
}
