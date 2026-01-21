// enable only in release before the final build
// disables the console window on windows
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod client;
mod server;

fn main() {
    client::main().unwrap();
    client::main().unwrap();
}
