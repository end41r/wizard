use std::{collections::HashMap};

use rand::seq::SliceRandom;
use crate::gamelogic::{card::Suit, game_state::GameState, round::Round};

type Err = &'static str;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Game {
    pub rounds: Vec<Round>,
    pub players: Vec<usize>,
    pub started: bool,
    pub is_over: bool,
}

impl Game {
    pub fn new() -> Self {
        Game {
            rounds: vec![],
            players: vec![],
            started: false,
            is_over: false
        }
    }

    pub fn set_trump(&mut self, player_id: usize, suit: Suit) -> Result<GameState, Err> {
        if !self.started {
            return Err("Game has not started yet");
        }
        let current_round = self.rounds.last_mut().unwrap();
        match current_round.set_trump(player_id, suit) {
            Ok(_) => {
                Ok(self.current_game_state())
            },
            Err(e) => return Err(e),
        }
    }

    pub fn set_called(&mut self, player_id: usize, value: usize) -> Result<GameState, Err> {
        if !self.started {
            return Err("Game has not started yet");
        }
        let current_round = self.rounds.last_mut().unwrap();
        match current_round.set_called(player_id, value) {
            Ok(_) => {
                Ok(self.current_game_state())
            },
            Err(e) => return Err(e),
        }
    }

    pub fn play_card(&mut self, player_id: usize, card: crate::gamelogic::card::Card) -> Result<GameState, Err> {
        if !self.started {
            return Err("Game has not started yet");
        }
        let current_round = self.rounds.last_mut().unwrap();
        match current_round.play_card(player_id, card) {
            Ok(_) => {
                if current_round.is_over {
                    if  self.rounds.len() < self.total_rounds() - 1 {
                        println!("{:#?}", self.current_game_state());
                        self.start_new_round();
                    } else {
                        self.is_over = true;
                    }
                }
                Ok(self.current_game_state())
            },
            Err(e) => return Err(e),
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

    pub fn start(&mut self) -> Result<GameState, Err> {
        if self.players.len() < 3 {
            return Err("Need more than two players to start a game.")
        }
        if self.players.len() > 6 {
            return Err("Need less than seven players to start a game.")
        }
        self.players.shuffle(&mut rand::rng());
        self.started = true;
        self.start_new_round();
        Ok(self.current_game_state())
    }

    fn start_new_round(&mut self) {
        let next_round_number = self.rounds.len();
        println!("[Game] Starting a new round: {}", next_round_number);
        let new_round = Round::new(next_round_number.try_into().unwrap(), &self.players.clone());
        self.rounds.push(new_round);
    }

    fn current_game_state(&self) -> GameState {
        let players = self
            .players
            .iter()
            .cloned()
            .map(|player| (player, self.rounds.iter().map(|round| round.players.get(&player).unwrap().points).sum()))
            .collect::<HashMap<usize, i32>>();
        GameState {
            current_round: self.rounds.last().unwrap().to_state(),
            total_rounds: self.total_rounds(),
            players,
            over: self.is_over,
        }
    }

    fn total_rounds(&self) -> usize {
        60 / self.players.len()
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

