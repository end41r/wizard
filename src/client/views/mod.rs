mod button;
mod gameplay;
mod lobby;
mod menu;
mod utils;

pub use button::{Button, ButtonMessage};

use iced::Element;

use super::{App, AppMessage, MenuState};

pub fn view(state: &App) -> Element<'_, AppMessage> {
    match state.menu {
        MenuState::Main => menu::view_main_menu(state),
        MenuState::Host => menu::view_host_menu(state),
        MenuState::Join => menu::view_join_menu(state),
        MenuState::Rules => menu::view_rules_menu(state),
        MenuState::Lobby => lobby::view_lobby_menu(state),
        MenuState::Playing => gameplay::view_gameplay(state),
        MenuState::PlayingTest => gameplay::view_test_gameplay(state),
    }
}
