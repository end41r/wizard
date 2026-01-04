use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GameState {
    pub current_round: usize,
    pub total_rounds: usize,
    pub players: HashMap<usize, usize>
}
