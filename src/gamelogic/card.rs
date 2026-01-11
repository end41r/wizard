use strum_macros::EnumIter;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum Symbol {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Wizard,
    Jester
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Card {
    pub suit: Suit,
    pub symbol: Symbol
}

impl Card {
    pub fn new(suit: Suit, symbol: Symbol) -> Self {
        Self { suit, symbol }
    }
}