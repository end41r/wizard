// Enable only in release before the final build.
// Disables the console window on Windows.
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod client;
mod server;

fn main() {
    client::main().unwrap();
}
