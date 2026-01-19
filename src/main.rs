mod api;
mod client;
mod game_elements;
mod server;

fn main() {
    client::main().unwrap();
}