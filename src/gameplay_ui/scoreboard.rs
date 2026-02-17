use std::collections::HashMap;

use iced::{
    Border, Color, Element, Length::Shrink, Size, Task, widget::{Column, Container, container, row, text}
};

use crate::{
    api::{Lobby, PlayerId},
    client::AppMessage,
    gameplay_ui::{GameViewMessage, SCOREBOARD_WIDTH_MUTL_WITH_WINDOW_WIDTH},
    ui_element_traits::{Message, Notifiable, Resizable, Viewable},
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

#[derive(Clone, Debug)]
pub enum ScoreBoardMessage {
    Update(ScoreBoardInfo),
}

impl Message for ScoreBoardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::ScoreBoardMessage(msg))
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
        &self,
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
                    .size(if is_header { self.size_small() } else { self.size_middle() })
                    .color(text_color),
            )
            .width(iced::Length::FillPortion(1))
            .padding(4)
        };

        let name_cell = |content: String| {
            container(
                text(content)
                    .size(if is_header { self.size_small() } else { self.size_middle() })
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
        .width(self.width());

        container(row_content)
            .width(self.width())
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

    fn size_small(&self) -> f32 {  // 10
        self.width() / 22.0
    }
    fn size_middle(&self) -> f32 {  // 11
        self.width() / 19.0
    }
    fn size_big(&self) -> f32 {  // 12
        self.width() / 16.0
    }
    fn size_huge(&self) -> f32 { // 18
        self.width() / 10.0
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

impl Resizable for ScoreBoard {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size
    }
    fn width(&self) -> f32 {
        SCOREBOARD_WIDTH_MUTL_WITH_WINDOW_WIDTH * self.window_size.width
    }
    /// The height of this item is dynamically inferenced
    /// so this function does not make sense for the scoreboard.
    fn height(&self) -> f32 {
        0.0
    }
}

impl Viewable for ScoreBoard {
    // AI Usage: overwrite the view so that the scoreboard is placed correctly
    // and uses rows+cells instead of rows+format strings
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut scores_col = Column::new().spacing(2);

        // Title
        scores_col = scores_col.push(
            container(text("Scoreboard").size(self.size_huge()).color(Color::WHITE))
                .padding(5),
        );

        scores_col = scores_col.push(
            container(
                text(format!("Round {}", self.info.round_number + 1))
                    .size(self.size_big())
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            )
            .padding([0, 5]),
        );

        // Header row
        scores_col = scores_col.push(self.scoreboard_row(
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

            scores_col = scores_col.push(self.scoreboard_row(
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
                    .size(self.size_small())
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            )
            .padding([8, 0]),
        );

        // Wrap in a styled container
        container(scores_col)
            .width(self.width())
            .height(Shrink)
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
    }
}
