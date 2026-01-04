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

    pub fn add_player(&mut self) -> usize {
        let id = self.players.len();
        self.players.push(id);
        id
    }
}

#[test]
fn create_a_new_game_with_one_player() {
    let mut game = Game::new();
    game.add_player();
}
