use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GameState {
    pub current_round: RoundState,
    pub total_rounds: usize,
    pub players: HashMap<usize, usize>
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RoundState {
    pub round_number: usize,
    pub player_states: HashMap<usize, PlayerState>,
    pub dealer: usize,
    pub current_player: usize,
    pub trump: Option<crate::gamelogic::card::Suit>,
    pub dealer_needs_to_set_trump: bool,
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlayerState {
    pub hand: Vec<crate::gamelogic::card::Card>,
    pub called: usize,
    pub tricks_won: usize,
}