# Gamelogic API

```rs
// Create a new Game. No parameters required
let game = Game::new()

// Add a new player. Names need to be stored somewhere else. Returns player id.
let player_id = game.add_player()

// Start the game. Returns infos about the current state of the game.
let game_state = game.start()

// Check if the trump needs to be set.
games_state.dealer_needs_to_set_Trump()

// Player Actions.

// Only the dealer can set the trump. To set the trump
game.set_trump(dealer, suit)

// To make a call for a Player.
game.set_called(player_id, value)

// To play a card for player 1 and the first card of his hand.
game.play_card(player_id, card)
