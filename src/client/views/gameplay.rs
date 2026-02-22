use iced::{
    alignment,
    widget::{
        button, column, container, row, scrollable, stack, text, text_input, Column, Image, Row,
    },
    Border, Color, ContentFit, Element,
};
use super::utils::{format_card, get_player_name};
use crate::api::{Suit};
use crate::client::{App, AppMessage};

// AI usage: write this function for testing purposes as a placeholder
pub fn display_trump_card<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if let Some(trump) = state.trump {
        let img_path = state.get_card_image(trump);
        container(
            Image::new(img_path)
                .width(100)
                .height(150)
        )
        .width(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .padding([8, 0])
        .into()
    } else {
        let img_path = "assets/cards/variations/back.png";
        container(
            Image::new(img_path)
                .width(100)
                .height(150)
        )
        .width(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .padding([8, 0])
        .into()
    }
}

pub fn view_gameplay<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let scoreboard = container(view_scoreboard(state))
        .width(iced::Length::Fixed(350.0))
        .height(iced::Length::Fill)
        .align_y(alignment::Vertical::Center)
        .padding([24.0, 24.0]);

    let main_content = container(Column::new())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Top);

    let trump_card = display_trump_card(state);
    let hand_display = container(build_hand_section_dbg(state))
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink)
        .align_x(alignment::Horizontal::Center)
        .padding([24.0, 24.0]);

    let left_panel = Column::new()
        .push(main_content)
        .push(trump_card)
        .push(hand_display)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

    let content = row![left_panel, scoreboard].height(iced::Length::Fill);

    stack![
        Image::new(state.img_ingame_background.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(ContentFit::Cover),
        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill),
    ]
    .into()
}

fn build_bidding_panel<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.is_bidding_phase || !state.is_my_turn || state.must_set_trump {
        return text("").into();
    }

    let max_bid = state.round_number + 1;

    let panel = column![
        text("Bid:").size(16).color(Color::from_rgb(1.0, 1.0, 1.0)),
        row![
            text_input("Enter bid", &state.bid_input)
                .on_input(AppMessage::BidInputChanged)
                .width(80),
            state.btn_submit_bid.view().padding(0),
        ]
        .spacing(6),
        text(format!("(0 to {max_bid})"))
            .size(12)
            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
    ]
    .spacing(6);

    container(panel).padding([8, 0]).into()
}


fn build_trump_panel<'a>(state: &'a App) -> Element<'a, AppMessage> {
    // Show only if dealer and must_set_trump
    if !state.must_set_trump || state.dealer != state.my_id {
        return text("").into();
    }

    let panel = column![
        text("Select Trump Suit:").size(16).color(Color::from_rgb(1.0, 1.0, 1.0)),
        row![
            button(Image::new("assets/cards/variations/red_1.png"))
                .width(80)
                .height(120)
                .on_press(AppMessage::SetTrump(Suit::Red))
                .padding(0),
            button(Image::new("assets/cards/variations/green_1.png"))
                .width(80)
                .height(120)
                .on_press(AppMessage::SetTrump(Suit::Green))
                .padding(0),
            button(Image::new("assets/cards/variations/blue_1.png"))
                .width(80)
                .height(120)
                .on_press(AppMessage::SetTrump(Suit::Blue))
                .padding(0),
            button(Image::new("assets/cards/variations/yellow_1.png"))
                .width(80)
                .height(120)
                .on_press(AppMessage::SetTrump(Suit::Yellow))
                .padding(0),
        ]
        .spacing(6),
    ]
    .spacing(6);

    container(panel).padding([8, 0]).into()
}
// AI Usage: overwrite the view so that the scoreboard is placed correctly
// and uses rows+cells instead of rows+format strings
pub fn view_scoreboard<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let mut scores_col = Column::new().spacing(2);

    scores_col = scores_col.push(
        container(text("Scoreboard").size(18).color(Color::WHITE))
            .width(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .padding(5),
    );

    scores_col = scores_col.push(
        container(
            text(format!("Round {}", state.round_number + 1))
                .size(12)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
        )
        .width(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .padding([0, 5]),
    );

    scores_col = scores_col.push(scoreboard_row("Name", "Pkt", "Won", "Bid", true, false));

    for player_id in &state.player_order {
        let mut player_name = get_player_name(state, *player_id);
        let score = state.scores.get(player_id).unwrap_or(&0);
        let tricks = state.tricks_won.get(player_id).unwrap_or(&0);
        let bid = state.bids.get(player_id).unwrap_or(&0);
        let is_self = state.my_id == Some(*player_id);

        if player_name.is_empty() {
            player_name = "???".to_string();
        }

        scores_col = scores_col.push(scoreboard_row(
            &player_name,
            &score.to_string(),
            &tricks.to_string(),
            &bid.to_string(),
            false,
            is_self,
        ));
    }

    scores_col = scores_col.push(
        container(
            text("Bids for current round")
                .size(10)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
        )
        .width(iced::Length::Fill)
        .center_x(iced::Length::Fill)
        .padding([8, 0]),
    );

    if state.must_set_trump && state.dealer == state.my_id {
        scores_col = scores_col.push(build_trump_panel(state)).align_x(iced::Center);
    } else if !state.must_set_trump {
        scores_col = scores_col.push(build_bidding_panel(state)).align_x(iced::Center);
    }

    container(scores_col)
        .width(iced::Length::Fill)
        .padding(10)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.6).into()),
            border: Border {
                color: Color::from_rgba(1.0, 0.85, 0.4, 0.5),
                width: 2.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn scoreboard_row(
    name: &str,
    score: &str,
    tricks: &str,
    bid: &str,
    is_header: bool,
    is_self: bool,
) -> Element<'static, AppMessage> {
    let text_color = if is_self {
        Color::from_rgb(1.0, 0.85, 0.4)
    } else {
        Color::WHITE
    };

    let cell = |content: String| {
        container(
            text(content)
                .size(if is_header { 10 } else { 11 })
                .color(text_color),
        )
        .width(iced::Length::FillPortion(1))
        .center_x(iced::Length::Fill)
        .padding(4)
    };

    let name_cell = |content: String| {
        container(
            text(content)
                .size(if is_header { 10 } else { 11 })
                .color(text_color),
        )
        .width(iced::Length::FillPortion(2))
        .padding(4)
    };

    let row_content = row![
        name_cell(name.to_string()),
        cell(score.to_string()),
        cell(tricks.to_string()),
        cell(bid.to_string()),
    ]
    .width(iced::Length::Fill);

    container(row_content)
        .width(iced::Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(if is_self {
                Color::from_rgba(1.0, 0.85, 0.4, 0.15).into()
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, if is_header { 0.4 } else { 0.2 }).into()
            }),
            border: Border {
                color: if is_self {
                    Color::from_rgba(1.0, 0.85, 0.4, 0.5)
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                },
                width: if is_self { 1.5 } else { 1.0 },
                radius: 2.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// =======================================
/// EASTER EGG SECTION: DEBUG GAMEPLAY VIEW
/// =======================================
pub fn view_test_gameplay<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let content = column![
        text("WIZARD").size(24),
        build_game_over_section_dbg(state),
        build_header_dbg(state),
        build_players_section_dbg(state),
        build_trump_section_dbg(state),
        build_bidding_section_dbg(state),
        build_bids_display_dbg(state),
        build_trick_display_dbg(state),
        build_hand_section_dbg(state),
        build_scores_section_dbg(state),
        build_log_section_dbg(state),
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

fn build_header_dbg<'a>(state: &'a App) -> Column<'a, AppMessage> {
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

    let phase = if state.must_set_trump {
        "Set Trump"
    } else if state.is_bidding_phase {
        "Bidding"
    } else {
        "Playing"
    };

    column![
        text(format!(
            "Round {} | Trump: {} | You: {}",
            state.round_number, trump_str, my_name
        ))
        .size(18),
        text(format!(
            "Current Player: {} | Phase: {}",
            current_player_name, phase
        ))
        .size(14),
        text(format!("Status: {}", state.last_msg)).size(12),
    ]
    .spacing(5)
}

fn build_trump_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.must_set_trump {
        return text("").into();
    }

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
}

fn build_bidding_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.is_bidding_phase || !state.is_my_turn || state.must_set_trump {
        return text("").into();
    }

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
}

fn build_bids_display_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if state.bids.is_empty() {
        return text("").into();
    }

    let mut bids_col = Column::new().spacing(2);
    bids_col = bids_col.push(text("Tricks / Bids:").size(14));

    for (player_id, bid) in &state.bids {
        let player_name = get_player_name(state, *player_id);
        let tricks = state.tricks_won.get(player_id).unwrap_or(&0);
        bids_col = bids_col.push(text(format!("  {}: {} / {}", player_name, tricks, bid)).size(12));
    }

    bids_col.into()
}

fn build_trick_display_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if state.current_trick.is_empty() {
        return text("Trick: (empty)").size(12).into();
    }

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
}

fn build_hand_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let mut hand_col = Column::new().spacing(5);
    hand_col = hand_col.push(text(format!("Your Hand ({} cards):", state.hand.len())).size(16));

    let mut card_row = Row::new().spacing(5);
    for card in &state.hand {
        let is_valid = state.valid_cards.is_empty() || state.valid_cards.contains(card);
        let can_play =
            state.is_my_turn && !state.is_bidding_phase && !state.must_set_trump && is_valid;

        println!("Card: {}, Valid: {}, Can Play: {}", format_card(card), is_valid, can_play);
        let img_handle = state.get_card_image(*card).clone();

        let card_btn = if can_play {
            stack![
                button(text("").size(11))
                    .on_press(AppMessage::PlayCard(*card))
                    .padding(8)
                    .width(80)
                    .height(120),
                Image::new(img_handle)
                    .width(80)
                    .height(120),
            ]
        } else {
            stack![Image::new(img_handle)
                .width(80)
                .height(120)
                .opacity(0.5),]
        };
        card_row = card_row.push(card_btn);
    }

    hand_col = hand_col.push(
        scrollable(card_row).direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        )),
    );

    hand_col.into()
}

fn build_scores_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let mut scores_col = Column::new().spacing(2);
    scores_col = scores_col.push(text("Scores:").size(14));

    for player_id in &state.player_order {
        let player_name = get_player_name(state, *player_id);
        let score = state.scores.get(player_id).unwrap_or(&0);
        scores_col = scores_col.push(text(format!("  {}: {} pts", player_name, score)).size(12));
    }

    scores_col.into()
}

fn build_log_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let mut log_col = Column::new().spacing(2);
    log_col = log_col.push(text("Game Log:").size(14));

    let start = state.game_log.len().saturating_sub(15);
    for entry in state.game_log.iter().skip(start) {
        log_col = log_col.push(text(entry).size(10));
    }

    scrollable(log_col).height(150).into()
}

fn build_players_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if state.player_order.is_empty() {
        return text("").into();
    }

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
}

fn build_game_over_section_dbg<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.game_over {
        return text("").into();
    }

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
        state.btn_back_to_menu.view().padding(8),
    ]
    .spacing(10)
    .into()
}
