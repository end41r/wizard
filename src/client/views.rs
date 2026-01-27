use iced::{
    widget::{
        button, column, container, pick_list, row, scrollable, text, text_input, Column, Row,
    },
    Element,
};

use super::{App, AppMessage, MenuState, PlayerCount};
use crate::api::{Card, Suit, Value};

/// Format a card for display (e.g., "5 Red", "Wizard", "Jester")
fn format_card(card: &Card) -> String {
    let value_str = match card.value {
        Value::Jester => "Jester".to_string(),
        Value::Wizard => "Wizard".to_string(),
        Value::Number(n) => n.to_string(),
    };

    match card.value {
        Value::Jester | Value::Wizard => value_str,
        Value::Number(_) => format!("{}\n{:?}", value_str, card.suit),
    }
}

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

fn get_player_name(state: &App, player_id: u64) -> String {
    if state.my_id == Some(player_id) {
        return "You".to_string();
    }
    if let Some(ref lobby) = state.lobby {
        if let Some(player) = lobby.players.iter().find(|p| p.id == player_id) {
            return player.name.clone();
        }
    }
    format!("Player {}", player_id)
}

fn view_gameplay<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let my_name = state
        .my_id
        .map(|id| get_player_name(state, id))
        .unwrap_or("?".to_string());
    let trump_str = state
        .trump
        .map(|s| format!("{:?}", s))
        .unwrap_or("None".to_string());
    let current_player_name = state
        .current_player
        .map(|id| get_player_name(state, id))
        .unwrap_or("?".to_string());

    let header = column![
        text(format!(
            "Round {} | Trump: {} | You: {}",
            state.round_number, trump_str, my_name
        ))
        .size(18),
        text(format!(
            "Current Player: {} | Phase: {}",
            current_player_name,
            if state.must_set_trump {
                "Set Trump"
            } else if state.is_bidding_phase {
                "Bidding"
            } else {
                "Playing"
            }
        ))
        .size(14),
        text(format!("Status: {}", state.last_msg)).size(12),
    ]
    .spacing(5);

    // Trump selection (if dealer needs to set)
    let trump_section: Element<'a, AppMessage> = if state.must_set_trump {
        column![
            text("SELECT TRUMP SUIT:").size(16),
            row![
                button("Red")
                    .on_press(AppMessage::SetTrump(Suit::Red))
                    .padding(8),
                button("Blue")
                    .on_press(AppMessage::SetTrump(Suit::Blue))
                    .padding(8),
                button("Green")
                    .on_press(AppMessage::SetTrump(Suit::Green))
                    .padding(8),
                button("Yellow")
                    .on_press(AppMessage::SetTrump(Suit::Yellow))
                    .padding(8),
            ]
            .spacing(5),
        ]
        .spacing(5)
        .into()
    } else {
        text("").into()
    };

    // Bidding section
    let bidding_section: Element<'a, AppMessage> =
        if state.is_bidding_phase && state.is_my_turn && !state.must_set_trump {
            column![
                text("YOUR BID:").size(16),
                row![
                    text_input("Enter bid", &state.bid_input)
                        .on_input(AppMessage::BidInputChanged)
                        .width(80),
                    button("Submit Bid")
                        .on_press(AppMessage::SubmitBid)
                        .padding(8),
                ]
                .spacing(5),
                text(format!("(0 to {})", state.round_number + 1)).size(12),
            ]
            .spacing(5)
            .into()
        } else {
            text("").into()
        };

    let bids_display: Element<'a, AppMessage> = if !state.bids.is_empty() {
        let mut bids_col = Column::new().spacing(2);
        bids_col = bids_col.push(text("Tricks / Bids:").size(14));
        for (player_id, bid) in &state.bids {
            let player_name = get_player_name(state, *player_id);
            let tricks = state.tricks_won.get(player_id).unwrap_or(&0);
            bids_col =
                bids_col.push(text(format!("  {}: {} / {}", player_name, tricks, bid)).size(12));
        }
        bids_col.into()
    } else {
        text("").into()
    };

    let trick_display: Element<'a, AppMessage> = if !state.current_trick.is_empty() {
        let mut trick_col = Column::new().spacing(2);
        trick_col = trick_col.push(text("Current Trick:").size(14));
        for (player_id, card) in &state.current_trick {
            let player_name = get_player_name(state, *player_id);
            let card_str = format_card(card);
            trick_col = trick_col.push(
                text(format!(
                    "  {} played {}",
                    player_name,
                    card_str.replace('\n', " ")
                ))
                .size(12),
            );
        }
        trick_col.into()
    } else {
        text("Trick: (empty)").size(12).into()
    };

    let hand_section: Element<'a, AppMessage> = {
        let mut hand_col = Column::new().spacing(5);
        hand_col =
            hand_col.push(text(format!("Your Hand ({} cards):", state.my_hand.len())).size(16));

        let mut card_row = Row::new().spacing(5);
        for card in &state.my_hand {
            let card_text = format_card(card);
            let is_valid = state.valid_cards.is_empty() || state.valid_cards.contains(card);
            let can_play =
                state.is_my_turn && !state.is_bidding_phase && !state.must_set_trump && is_valid;

            let card_btn = if can_play {
                button(text(card_text).size(11))
                    .on_press(AppMessage::PlayCard(*card))
                    .padding(8)
            } else {
                button(text(card_text).size(11)).padding(8)
            };
            card_row = card_row.push(card_btn);
        }
        hand_col = hand_col.push(scrollable(card_row).direction(
            scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
        ));
        hand_col.into()
    };

    let scores_section: Element<'a, AppMessage> = {
        let mut scores_col = Column::new().spacing(2);
        scores_col = scores_col.push(text("Scores:").size(14));
        for player_id in &state.player_order {
            let player_name = get_player_name(state, *player_id);
            let score = state.scores.get(player_id).unwrap_or(&0);
            scores_col =
                scores_col.push(text(format!("  {}: {} pts", player_name, score)).size(12));
        }
        scores_col.into()
    };

    let game_over_section: Element<'a, AppMessage> = if state.game_over {
        let winner_name = state
            .winner
            .map(|id| get_player_name(state, id))
            .unwrap_or("Unknown".to_string());
        let is_me = state.my_id == state.winner;
        column![
            text(if is_me {
                "🎉 YOU WON! 🎉"
            } else {
                "GAME OVER"
            })
            .size(24),
            text(format!("Winner: {}", winner_name)).size(18),
            button("Back to Menu")
                .on_press(AppMessage::BackToMenu)
                .padding(10),
        ]
        .spacing(10)
        .into()
    } else {
        text("").into()
    };

    let log_section: Element<'a, AppMessage> = {
        let mut log_col = Column::new().spacing(2);
        log_col = log_col.push(text("Game Log:").size(14));
        let start = if state.game_log.len() > 15 {
            state.game_log.len() - 15
        } else {
            0
        };
        for entry in state.game_log.iter().skip(start) {
            log_col = log_col.push(text(entry).size(10));
        }
        scrollable(log_col).height(150).into()
    };

    let players_section: Element<'a, AppMessage> = if !state.player_order.is_empty() {
        let mut players_str = String::from("Players: ");
        for (i, pid) in state.player_order.iter().enumerate() {
            let is_current = state.current_player == Some(*pid);
            let player_name = get_player_name(state, *pid);
            players_str.push_str(&format!(
                "{}{}{}",
                if is_current { "[" } else { "" },
                player_name,
                if is_current { "]" } else { "" }
            ));
            if i < state.player_order.len() - 1 {
                players_str.push_str(" → ");
            }
        }
        text(players_str).size(12).into()
    } else {
        text("").into()
    };

    let content = column![
        text("WIZARD").size(24),
        game_over_section,
        header,
        players_section,
        trump_section,
        bidding_section,
        bids_display,
        trick_display,
        hand_section,
        scores_section,
        log_section,
        button("Back to Menu")
            .on_press(AppMessage::BackToMenu)
            .padding(8),
    ]
    .spacing(10)
    .padding(20);

    container(scrollable(content))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}
