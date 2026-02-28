// Disables the console widndow on Windows.
#![cfg_attr(not(feature = "wiz_debug"), windows_subsystem = "windows")]

mod animation;
mod api;
mod client;
pub mod gamelogic;
mod gameplay_ui;
mod server;
mod ui_element_traits;

fn main() {
    client::main().unwrap();
}
