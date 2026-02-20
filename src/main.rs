// Enable only in release before the final build.
// Disables the console window on Windows.
#![cfg_attr(feature = "wiz_debug", windows_subsystem = "windows")]

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
