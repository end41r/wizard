use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column},
    Element,
};

use super::{App, AppMessage, MenuState, PlayerCount};

pub fn view(state: &App) -> Element<'_, AppMessage> {
    match state.menu {
        MenuState::Main => view_main_menu(state),
        MenuState::Host => view_host_menu(state),
        MenuState::Join => view_join_menu(state),
        MenuState::Rules => view_rules_menu(),
        MenuState::Lobby => view_lobby_menu(state),
        MenuState::Playing => view_gameplay(state),
    }
}

fn view_main_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let content = column![
        text("Wizard - Main Menu").size(40),
        button("Host").on_press(AppMessage::Host).padding(10),
        button("Join")
            .on_press(AppMessage::Navigate(MenuState::Join))
            .padding(10),
        button("Gamerules")
            .on_press(AppMessage::GameRules)
            .padding(10),
        button("Exit Game")
            .on_press(AppMessage::CloseGame)
            .padding(10),
        text(state.last_msg.clone()),
    ]
    .spacing(20)
    .align_x(iced::alignment::Horizontal::Center);

    container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

fn view_host_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let count_options = vec![
        PlayerCount::P3,
        PlayerCount::P4,
        PlayerCount::P5,
        PlayerCount::P6,
    ];
    let can_join = !state.host_name.is_empty();
    let content = column![
        text("Host").size(30),
        row![
            text(&state.ip),
            button("copy").on_press(AppMessage::CopyToClipboard(state.ip.clone())),
        ]
        .spacing(10),
        text("Name:"),
        text_input("Your Name", &state.host_name).on_input(AppMessage::HostNameChanged),
        text("Player Count:"),
        pick_list(
            count_options.clone(),
            Some(state.host_player_count),
            AppMessage::HostPlayerCountChanged
        ),
        button("Create Lobby").on_press_maybe(if can_join {
            Some(AppMessage::CreateLobby)
        } else {
            None
        }),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    container(content)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

fn view_join_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let can_join = !state.ip.is_empty() && !state.join_name.is_empty();
    let content = column![
        text("Join").size(30),
        text("Name:"),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        button("Connect").on_press_maybe(if can_join {
            Some(AppMessage::Connect)
        } else {
            None
        }),
        text("Progress:"),
        text(&state.last_msg),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    container(content)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

fn view_rules_menu<'a>() -> Element<'a, AppMessage> {
    let content = column![
        text("Game Rules").size(30),
        text("Here are the game rules (placeholder)."),
        button("Back").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .align_x(iced::alignment::Horizontal::Left);

    container(content)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

fn view_lobby_menu<'a>(state: &App) -> Element<'a, AppMessage> {
    if !state.connected {
        return container(column![
            text("Nicht verbunden zum Server. / IP wurde falsch eingegeben"),
            button("Zurück").on_press(AppMessage::BackToMenu)
        ])
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into();
    }
    if let Some(lobby) = &state.lobby {
        let mut player_rows = Column::new().spacing(10);
        for p in &lobby.players {
            let ready_text = if p.ready { "Bereit" } else { "Nicht bereit" };
            let toggle = button(ready_text).on_press_maybe(if Some(p.id) == state.my_id {
                Some(AppMessage::ToggleReady(p.id))
            } else {
                None
            });
            let row = row![
                text(format!(
                    "{}{}",
                    if p.is_host { "(Host) " } else { "" },
                    p.name
                )),
                toggle
            ];
            player_rows = player_rows.push(row);
        }

        let mut chat_block = Column::new().spacing(5);
        for (sender, msg) in &lobby.chat {
            chat_block = chat_block.push(text(format!("{}: {}", sender, msg)));
        }

        let can_start = lobby.players.len() == state.host_player_count.to_usize()
            && lobby.players.iter().all(|p| p.ready);
        let start_button = row![
            button("Starten").on_press_maybe(
                if can_start
                    && state.my_id.is_some()
                    && state.my_id.unwrap()
                        == lobby
                            .players
                            .iter()
                            .find(|p| p.is_host)
                            .map(|p| p.id)
                            .unwrap_or_default()
                {
                    Some(AppMessage::StartGame)
                } else {
                    None
                }
            ),
            text(if !can_start {
                " (Warten auf Spieler...)"
            } else if state.my_id.is_some()
                && state.my_id.unwrap()
                    != lobby
                        .players
                        .iter()
                        .find(|p| p.is_host)
                        .map(|p| p.id)
                        .unwrap_or_default()
            {
                " (Nur der Host kann starten)"
            } else {
                ""
            })
        ]
        .spacing(5);

        let content = column![
            text("Lobby").size(30),
            row![
                text("Host Address:"),
                text_input("Address to share", &state.ip)
            ]
            .spacing(10),
            text(format!(
                "Spieler: {}/{}",
                lobby.players.len(),
                state.host_player_count.to_usize()
            )),
            player_rows,
            scrollable(chat_block).height(150).width(400),
            row![
                text_input("Nachricht", &state.chat_input).on_input(AppMessage::ChatInputChanged),
                button("Senden").on_press(AppMessage::SendChat),
            ],
            start_button,
            button("Zurück zum Menü").on_press(AppMessage::BackToMenu)
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .center_x(iced::Fill)
            .center_y(iced::Fill)
            .into()
    } else {
        container(column![
            text("Keine Lobby vorhanden"),
            button("Zurück").on_press(AppMessage::BackToMenu)
        ])
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
    }
}

fn view_gameplay<'a>(_state: &App) -> Element<'a, AppMessage> {
    let content = column![
        text("Gameplay Screen").size(30),
        text("Game in progress... (placeholder)"),
        button("Zurück zum Menü").on_press(AppMessage::BackToMenu),
    ]
    .spacing(10)
    .padding(20)
    .align_x(iced::alignment::Horizontal::Left);

    container(content)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}
