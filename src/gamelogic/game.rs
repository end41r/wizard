use rand::seq::SliceRandom;

type Err = &'static str;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Game {
    players: Vec<usize>,
    started: bool,
}

impl Game {
    pub fn new() -> Self {
        Game { 
            players: vec![],
            started: false,
        }
    }

    pub fn add_player(&mut self) -> Result<usize, Err> {
        if self.started {
            return Err("Cannot add a player after the game started")
        }
        let id = self.players.len();
        self.players.push(id);
        Ok(id)
    }

    pub fn remove_player(&mut self, id: usize) -> Result<(), Err> {
        if self.started {
            return Err("Cannot add a player after the game started")
        }
       
        let index_result = self.players.iter().position(|x| *x == id);
        match index_result {
            Some(index) => {
                self.players.remove(index);
                Ok(())
            },
            None => Ok(())
        }
    }

    pub fn start(&mut self) -> Result<(), Err> {
        if self.players.len() < 3 {
            return Err("Need more than two players to start a game.")
        }
        if self.players.len() > 6 {
            return Err("Need less than seven players to start a game.")
        }
        self.players.shuffle(&mut rand::rng());
        self.started = true;
        Ok(())
    }
}

#[test]
fn start_game_with_2_players() {
    let mut game = Game::new();
    let _ = game.add_player();
    let _ = game.add_player();
    assert!(game.start().is_err());
}

#[cfg(test)]
fn new_game_with_3_players() -> Game {
    let mut game = Game::new();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    game
}


#[test]
fn start_game_with_3_players() {
    let mut game = new_game_with_3_players();
    assert_eq!(game.started, false);
    assert!(game.start().is_ok());
    assert_eq!(game.players.len(), 3);
    assert_eq!(game.started, true);
}

#[test]
fn start_game_with_7_players() {
    let mut game = Game::new();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    let _ = game.add_player();
    assert!(game.start().is_err());
}

#[test]
fn add_player_after_game_start() {
    let mut game = new_game_with_3_players();
    let _ = game.start();
    assert!(game.add_player().is_err());
}

#[test]
fn remove_player_that_does_not_exist() {
    let mut game = new_game_with_3_players();
    assert!(game.remove_player(100).is_ok());
}

#[test]
fn remove_player_after_game_start() {
    let mut game = new_game_with_3_players();
    let i_wont_play = game.add_player();
    let _ = game.start();
    assert!(game.remove_player(i_wont_play.unwrap()).is_err());
}    

