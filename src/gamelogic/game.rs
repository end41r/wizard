#[derive(Clone, PartialEq, Eq, Debug)]

pub struct Game {
    players: Vec<usize>,
}

impl Game {
    pub fn new() -> Self {
        Game { 
            players: vec![],
        }
    }
}

#[test]
fn create_a_new_game() {
    let _ = Game::new();
}