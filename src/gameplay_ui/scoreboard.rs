use std::collections::HashMap;

use iced::{
    widget::{container, row, text, Column, Container},
    Border, Color, Element, Size, Task,
};

use crate::{
    api::{Lobby, PlayerId},
    client::AppMessage,
    gameplay_ui::GameViewMessage,
    ui_element_traits::{Message, Notifiable, Viewable},
};

#[derive(Clone, Debug)]
pub struct ScoreBoardInfo {
    round_number: usize,
    player_order: Vec<PlayerId>,
    scores: HashMap<PlayerId, i32>,
    tricks_won: HashMap<PlayerId, usize>,
    bids: HashMap<PlayerId, usize>,
    my_id: Option<PlayerId>,
    lobby: Option<Lobby>,
}

impl Default for ScoreBoardInfo {
    fn default() -> Self {
        Self {
            round_number: 0,
            player_order: Vec::new(),
            scores: HashMap::new(),
            tricks_won: HashMap::new(),
            bids: HashMap::new(),
            my_id: None,
            lobby: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ScoreBoardMessage {
    Update(ScoreBoardInfo),
}

impl Message for ScoreBoardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::ScoreBoardMessage(msg))
    }
}

impl ScoreBoardInfo {
    pub fn new(
        round_number: usize,
        player_order: Vec<PlayerId>,
        scores: HashMap<PlayerId, i32>,
        tricks_won: HashMap<PlayerId, usize>,
        bids: HashMap<PlayerId, usize>,
        my_id: Option<PlayerId>,
        lobby: Option<Lobby>,
    ) -> Self {
        Self {
            round_number,
            player_order,
            scores,
            tricks_won,
            bids,
            my_id,
            lobby,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreBoard {
    window_size: Size,
    info: ScoreBoardInfo,
}

impl ScoreBoard {
    pub fn new(window_size: Size, info: ScoreBoardInfo) -> Self {
        Self { window_size, info }
    }

    fn scoreboard_row<'a>(
        name: &str,
        score: &str,
        tricks: &str,
        bid: &str,
        is_header: bool,
        is_self: bool,
    ) -> Container<'a, AppMessage> {
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
    }

    /// Get player name from ID using lobby data
    pub fn get_player_name(&self, player_id: PlayerId) -> String {
        if self.info.my_id == Some(player_id) {
            return "You".to_string();
        }
        if let Some(ref lobby) = self.info.lobby {
            if let Some(player) = lobby.players.iter().find(|p| p.id == player_id) {
                return player.name.clone();
            }
        }
        format!("Player {}", player_id)
    }

    // bid_panel_footer is no longer needed for gameplay view
    #[allow(dead_code)]
    fn bid_panel_footer<'a>() -> Option<Element<'a, AppMessage>> {
        Some(
            text("Bids are shown for the current round only.")
                .size(11)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7))
                .into(),
        )
    }
}

impl Notifiable for ScoreBoard {
    type OwnMessage = ScoreBoardMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            ScoreBoardMessage::Update(info) => {
                self.info = info;
            }
        }
        Task::none()
    }
}

impl Viewable for ScoreBoard {
    // AI Usage: overwrite the view so that the scoreboard is placed correctly
    // and uses rows+cells instead of rows+format strings
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut scores_col = Column::new().spacing(2);

        // Title
        scores_col = scores_col.push(
            container(text("Scoreboard").size(18).color(Color::WHITE))
                .width(iced::Length::Fill)
                .center_x(iced::Length::Fill)
                .padding(5),
        );

        scores_col = scores_col.push(
            container(
                text(format!("Round {}", self.info.round_number + 1))
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            )
            .width(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .padding([0, 5]),
        );

        // Header row
        scores_col = scores_col.push(Self::scoreboard_row(
            "Name", "Pkt", "Won", "Bid", true, false,
        ));

        for player_id in &self.info.player_order {
            let mut player_name = self.get_player_name(*player_id);
            let score = self.info.scores.get(player_id).unwrap_or(&0);
            let tricks = self.info.tricks_won.get(player_id).unwrap_or(&0);
            let bid = self.info.bids.get(player_id).unwrap_or(&0);
            let is_self = self.info.my_id == Some(*player_id);

            if player_name.is_empty() {
                player_name = "???".to_string();
            }

            scores_col = scores_col.push(Self::scoreboard_row(
                &player_name,
                &score.to_string(),
                &tricks.to_string(),
                &bid.to_string(),
                false,
                is_self,
            ));
        }

        // Footer note
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

        // Wrap in a styled container
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
            .width(iced::Length::Fixed(250.0))
            .height(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .padding(10)
    }
}
