use std::collections::HashMap;

use crate::api::{Card, PlayerId, Suit, Value};
use crate::gamelogic::{round::Round, GameEvent};
use rand::seq::SliceRandom;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Game {
    pub rounds: Vec<Round>,
    pub players: Vec<PlayerId>,
    pub started: bool,
    pub is_over: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            rounds: vec![],
            players: vec![],
            started: false,
            is_over: false,
        }
    }

    pub fn set_trump(
        &mut self,
        player_id: PlayerId,
        suit: Suit,
    ) -> Result<Vec<GameEvent>, &'static str> {
        if !self.started {
            return Err("Game has not started yet");
        }
        let round = self.rounds.last_mut().unwrap();
        round.set_trump(player_id, suit)?;

        let mut events = vec![GameEvent::TrumpSet {
            suit,
            by_dealer: player_id,
        }];

        let starting_player = round.order[0];
        events.push(GameEvent::BiddingStarted {
            starting_player,
            cards_per_player: round.round_number + 1,
        });
        events.push(GameEvent::BidRequest {
            player: starting_player,
            min: 0,
            max: round.round_number + 1,
        });

        Ok(events)
    }

    pub fn bid(
        &mut self,
        player_id: PlayerId,
        amount: usize,
    ) -> Result<Vec<GameEvent>, &'static str> {
        if !self.started {
            return Err("Game has not started yet");
        }
        let round = self.rounds.last_mut().unwrap();
        round.set_called(player_id, amount)?;

        let mut events = vec![GameEvent::BidMade {
            player: player_id,
            amount,
        }];

        if round.bidding_phase {
            let next_bidder = round.order[round.current_bidder_index];
            events.push(GameEvent::BidRequest {
                player: next_bidder,
                min: 0,
                max: round.round_number + 1,
            });
        } else {
            let bids: HashMap<PlayerId, usize> = round
                .players
                .iter()
                .map(|(id, p)| (*id, p.called))
                .collect();
            events.push(GameEvent::BiddingFinished { bids });

            let leader = round.current_player;
            events.push(GameEvent::TrickStarted { leader });
            events.push(GameEvent::TurnRequest {
                player: leader,
                valid_cards: self.valid_cards_for(leader),
            });
        }

        Ok(events)
    }

    pub fn play_card(
        &mut self,
        player_id: PlayerId,
        card: Card,
    ) -> Result<Vec<GameEvent>, &'static str> {
        if !self.started {
            return Err("Game has not started yet");
        }

        // Capture trick cards before play_card (which may clear them after completion)
        let trick_cards_before: Vec<(PlayerId, Card)> =
            self.rounds.last().unwrap().current_trick.clone();
        let trick_size_before = trick_cards_before.len();

        let round = self.rounds.last_mut().unwrap();
        round.play_card(player_id, card)?;

        let mut events = vec![GameEvent::CardPlayed {
            player: player_id,
            card,
        }];

        let trick_completed = trick_size_before == self.players.len() - 1;

        if trick_completed {
            let round = self.rounds.last().unwrap();
            let winner = round.current_player;
            // Combine cards from before with the card just played
            let mut trick_cards = trick_cards_before;
            trick_cards.push((player_id, card));
            events.push(GameEvent::TrickFinished {
                winner,
                cards: trick_cards,
            });

            if round.is_over {
                // Send cumulative total scores, not just this round's points
                let scores = self.total_scores();
                let tricks_won: HashMap<PlayerId, usize> = round
                    .players
                    .iter()
                    .map(|(id, p)| (*id, p.tricks_won))
                    .collect();
                events.push(GameEvent::RoundFinished { scores, tricks_won });

                if self.rounds.len() < self.total_rounds() {
                    self.start_new_round();
                    events.extend(self.emit_round_start_events());
                } else {
                    self.is_over = true;
                    let final_scores = self.total_scores();
                    let winner = *final_scores.iter().max_by_key(|(_, s)| *s).unwrap().0;
                    events.push(GameEvent::GameFinished {
                        final_scores,
                        winner,
                    });
                }
            } else {
                let leader = self.rounds.last().unwrap().current_player;
                events.push(GameEvent::TrickStarted { leader });
                events.push(GameEvent::TurnRequest {
                    player: leader,
                    valid_cards: self.valid_cards_for(leader),
                });
            }
        } else {
            let next = self.rounds.last().unwrap().current_player;
            events.push(GameEvent::TurnRequest {
                player: next,
                valid_cards: self.valid_cards_for(next),
            });
        }

        Ok(events)
    }

    pub fn add_player(&mut self, id: PlayerId) -> Result<(), &'static str> {
        if self.started {
            return Err("Cannot add a player after the game started");
        }
        self.players.push(id);
        Ok(())
    }

    pub fn remove_player(&mut self, id: PlayerId) -> Result<(), &'static str> {
        if self.started {
            return Err("Cannot remove a player after the game started");
        }
        if let Some(index) = self.players.iter().position(|x| *x == id) {
            self.players.remove(index);
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<Vec<GameEvent>, &'static str> {
        if self.players.len() < 3 {
            return Err("Need more than two players to start a game.");
        }
        if self.players.len() > 6 {
            return Err("Need less than seven players to start a game.");
        }
        self.players.shuffle(&mut rand::rng());
        self.started = true;
        self.start_new_round();

        let mut events = vec![GameEvent::GameStarted {
            players: self.players.clone(),
        }];
        events.extend(self.emit_round_start_events());

        Ok(events)
    }

    fn emit_round_start_events(&self) -> Vec<GameEvent> {
        let round = self.rounds.last().unwrap();
        let mut events = vec![GameEvent::RoundStarted {
            round: round.round_number,
            cards_per_player: round.round_number + 1,
            trump: round.trump,
        }];

        for (player_id, player) in &round.players {
            events.push(GameEvent::HandDealt {
                player: *player_id,
                cards: player.hand.clone(),
            });
        }

        if round.dealer_needs_to_set_trump {
            events.push(GameEvent::DealerMustSetTrump {
                dealer: round.dealer,
            });
        } else { // else we start bidding
            let starting_player = round.order[0];
            events.push(GameEvent::BiddingStarted {
                starting_player,
                cards_per_player: round.round_number + 1,
            });
            events.push(GameEvent::BidRequest {
                player: starting_player,
                min: 0,
                max: round.round_number + 1,
            });
        }

        events
    }

    fn start_new_round(&mut self) {
        let next_round_number = self.rounds.len();
        println!("[Game] Starting a new round: {}", next_round_number);
        self.rounds
            .push(Round::new(next_round_number, &self.players.clone()));
    }

    fn valid_cards_for(&self, player_id: PlayerId) -> Vec<Card> {
        let round = self.rounds.last().unwrap();
        let player = round.players.get(&player_id).unwrap();
        let hand = &player.hand;

        if round.current_trick.is_empty() {
            return hand.clone();
        }

        let lead_suit = round
            .current_trick
            .iter()
            .find(|(_, c)| c.value != Value::Jester && c.value != Value::Wizard)
            .map(|(_, c)| c.suit);

        let Some(lead) = lead_suit else {
            return hand.clone();
        };

        let lead_cards: Vec<Card> = hand
            .iter()
            .filter(|c| c.suit == lead && c.value != Value::Wizard && c.value != Value::Jester)
            .cloned()
            .collect();

        if lead_cards.is_empty() {
            hand.clone()
        } else {
            hand.iter()
                .filter(|c| c.suit == lead || c.value == Value::Wizard || c.value == Value::Jester)
                .cloned()
                .collect()
        }
    }

    fn total_scores(&self) -> HashMap<PlayerId, i32> {
        self.players
            .iter()
            .cloned()
            .map(|player| {
                let total: i32 = self
                    .rounds
                    .iter()
                    .map(|round| round.players.get(&player).unwrap().points)
                    .sum();
                (player, total)
            })
            .collect()
    }

    fn total_rounds(&self) -> usize {
        60 / self.players.len()
    }
}

#[test]
fn start_game_with_2_players() {
    let mut game = Game::new();
    let _ = game.add_player(111);
    let _ = game.add_player(222);
    assert!(game.start().is_err());
}

#[cfg(test)]
fn new_game_with_3_players() -> Game {
    let mut game = Game::new();
    let _ = game.add_player(111);
    let _ = game.add_player(222);
    let _ = game.add_player(333);
    game
}

#[test]
fn start_game_with_3_players() {
    let mut game = new_game_with_3_players();
    assert!(!game.started);
    assert!(game.start().is_ok());
    assert_eq!(game.players.len(), 3);
    assert!(game.started);
}

#[test]
fn start_game_with_7_players() {
    let mut game = Game::new();
    let _ = game.add_player(111);
    let _ = game.add_player(222);
    let _ = game.add_player(333);
    let _ = game.add_player(444);
    let _ = game.add_player(555);
    let _ = game.add_player(666);
    let _ = game.add_player(777);
    assert!(game.start().is_err());
}

#[test]
fn add_player_after_game_start() {
    let mut game = new_game_with_3_players();
    let _ = game.start();
    assert!(game.add_player(888).is_err());
}

#[test]
fn remove_player_that_does_not_exist() {
    let mut game = new_game_with_3_players();
    assert!(game.remove_player(100).is_ok());
}

#[test]
fn remove_player_after_game_start() {
    let mut game = new_game_with_3_players();
    let _ = game.add_player(999);
    let _ = game.start();
    assert!(game.remove_player(999).is_err());
}
