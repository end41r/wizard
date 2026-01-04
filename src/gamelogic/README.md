# Gamelogic API

```rs
// Create a new Game. No parameters required
let game = Game::new()

// Add a new player. Names need to be stored somewhere else. Returns player id.
let player_id = game.add_player()

// Start the game. Returns infos about the current state of the game.
let game_state = game.start()
```