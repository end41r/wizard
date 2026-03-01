//! Lobby view for waiting players before game start.

use iced::{
    ContentFit, Element,
    widget::{Column, Image, column, container, row, scrollable, stack, text, text_input},
};

use super::utils::menu_panel;
use crate::client::{App, AppMessage};
use crate::{
    api::{Lobby, TextColor},
    client::views::utils::back_button_footer,
};

pub fn view_lobby_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.connected {
        return view_not_connected(state);
    }

    let Some(lobby) = &state.lobby else {
        return view_no_lobby(state);
    };

    view_lobby_content(state, lobby)
}

fn view_not_connected<'a>(state: &'a App) -> Element<'a, AppMessage> {
    stack![
        Image::new(state.img_lobby_menu.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(ContentFit::Cover),
        container(column![
            text("Nicht verbunden zum Server. / IP wurde falsch eingegeben").white(),
            state.btn_back.view().padding(0)
        ])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
    ]
    .into()
}

fn view_no_lobby<'a>(state: &'a App) -> Element<'a, AppMessage> {
    stack![
        Image::new(state.img_lobby_menu.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(ContentFit::Cover),
        container(column![
            text("Keine Lobby vorhanden").white(),
            state.btn_back.view().padding(0)
        ])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .center_y(iced::Length::Fill)
    ]
    .into()
}

fn view_lobby_content<'a>(state: &'a App, lobby: &'a Lobby) -> Element<'a, AppMessage> {
    let player_rows = build_player_rows(state, lobby);
    let chat_block = build_chat_block(lobby);
    let start_button = build_start_button(state, lobby);

    let content = column![
        text("Lobby").size(30).white(),
        row![
            text("Host Address:"),
            text_input("Address to share", &state.ip)
        ]
        .spacing(10),
        text(format!(
            "Spieler: {}/{}",
            lobby.players.len(),
            state.host_player_count.to_usize()
        ))
        .white(),
        player_rows,
        scrollable(chat_block).height(150).width(400),
        row![
            text_input("Nachricht", &state.chat_input).on_input(AppMessage::ChatInputChanged),
            state.btn_send_chat.view().padding(0),
        ],
        start_button,
        state.btn_back_to_menu.view().padding(0)
    ]
    .spacing(10)
    .padding(20);

    stack![
        Image::new(state.img_lobby_menu.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(ContentFit::Cover),
        menu_panel(state, "Lobby", content.into(), back_button_footer(state))
    ]
    .into()
}

fn build_player_rows<'a>(state: &'a App, lobby: &'a Lobby) -> Column<'a, AppMessage> {
    let mut player_rows = Column::new().spacing(10);

    for p in &lobby.players {
        let ready_text = if p.ready { "Bereit" } else { "Nicht bereit" };
        let toggle = if Some(p.id) == state.my_id {
            state.btn_ready_owned.view_with_label(ready_text)
        } else {
            state.btn_ready_owned.view_disabled_with_label(ready_text)
        };

        let row = row![
            text(format!(
                "{}{}",
                if p.is_host { "(Host) " } else { "" },
                p.name
            ))
            .white(),
            toggle
        ];
        player_rows = player_rows.push(row);
    }

    player_rows
}

fn build_chat_block<'a>(lobby: &'a Lobby) -> Column<'a, AppMessage> {
    let mut chat_block = Column::new().spacing(5);

    for (sender, msg) in &lobby.chat {
        chat_block = chat_block.push(text(format!("{}: {}", sender, msg)).white());
    }

    chat_block
}

fn build_start_button<'a>(state: &'a App, lobby: &'a Lobby) -> iced::widget::Row<'a, AppMessage> {
    let can_start = if cfg!(feature = "wiz_debug") {
        true
    } else {
        lobby.players.len() == state.host_player_count.to_usize()
            && lobby.players.iter().all(|p| p.ready)
    };
    let host_id = lobby
        .players
        .iter()
        .find(|p| p.is_host)
        .map(|p| p.id)
        .unwrap_or_default();

    let i_am_host = state.my_id.is_some() && state.my_id.unwrap() == host_id;

    let start_button_view = if can_start && i_am_host {
        state.btn_start_game.view().padding(0)
    } else {
        state.btn_start_game.view_disabled().padding(0)
    };

    let status_text = if !can_start {
        " (Warten auf Spieler...)"
    } else if state.my_id.is_some() && !i_am_host {
        " (Nur der Host kann starten)"
    } else {
        ""
    };

    row![start_button_view, text(status_text).white()].spacing(5)
}
