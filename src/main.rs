use wizard::gamelogic::game::Game;

fn main() {
    // Create a new Game. No parameters required
    let mut game = Game::new();

    // Add a new player. Names need to be stored somewhere else. Returns player id.
    let _player_id = game.add_player();
    let _player_id_1 = game.add_player();
    let _player_id_2 = game.add_player();

    // Start the game. Returns infos about the current state of the game.
    let game_state = game.start();

    println!("{:?}", game_state)
}
