mod animation;
mod api;
mod client;
mod gameplay_ui;
mod server;
mod ui_element_traits;

fn main() {
    client::main().unwrap();
}