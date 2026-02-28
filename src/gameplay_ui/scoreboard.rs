use std::collections::HashMap;

use iced::{
    mouse::Interaction,
    widget::{
        column, container, image::FilterMethod, mouse_area, row, text, text_input, Column,
        Container, Image,
    },
    Border, Color,
    Length::Shrink,
    Size, Task,
};

use crate::animation::{Easing, ReversableBasicAnimation};
use derive_more::{Deref, DerefMut};

use crate::{
    api::{Lobby, PlayerId, Suit},
    client::{views::Button, AppMessage, TaskBatcher},
    gameplay_ui::{
        GameViewMessage, CARD_WIDTH_HEIGHT_RATIO, SCOREBOARD_WIDTH_MUTL_WITH_WINDOW_WIDTH,
    },
    ui_element_traits::{Animated, Message, Notifiable, ResizableDynHeight, Viewable},
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
    must_set_trump: bool,
    dealer: Option<PlayerId>,
    is_bidding_phase: bool,
    is_my_turn: bool,
    bid_input: String,
    current_player: Option<PlayerId>,
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
            must_set_trump: false,
            dealer: None,
            is_bidding_phase: false,
            is_my_turn: false,
            bid_input: String::new(),
            current_player: None,
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
        must_set_trump: bool,
        dealer: Option<PlayerId>,
        is_bidding_phase: bool,
        is_my_turn: bool,
        bid_input: String,
        current_player: Option<PlayerId>,
    ) -> Self {
        Self {
            round_number,
            player_order,
            scores,
            tricks_won,
            bids,
            my_id,
            lobby,
            must_set_trump,
            dealer,
            is_bidding_phase,
            is_my_turn,
            bid_input,
            current_player,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ScoreBoardMessage {
    Update(ScoreBoardInfo),
    SuitHovered(Suit),
    SuitNotHovered(Suit),
}

impl Message for ScoreBoardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        GameViewMessage::convert_msg_from(GameViewMessage::ScoreBoardMessage(msg))
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct HoverAnimation(ReversableBasicAnimation);

impl HoverAnimation {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }
    pub fn get_scale(&self) -> f32 {
        0.8 + 0.2 * self.progress(Easing::InOutSine)
    }
}

#[derive(Debug, Clone)]
pub struct ScoreBoard {
    window_size: Size,
    pub btn_submit_bid: Button,
    info: ScoreBoardInfo,
    showing_trump_select: bool,
    hover_animation_red: HoverAnimation,
    hover_animation_green: HoverAnimation,
    hover_animation_blue: HoverAnimation,
    hover_animation_yellow: HoverAnimation,
}

impl ScoreBoard {
    /// AI Usage: write this function to get player order sorted by score
    fn sorted_player_order_by_score(&self) -> Vec<PlayerId> {
        let mut players: Vec<PlayerId> = self.info.player_order.clone();
        players.sort_by_key(|pid| std::cmp::Reverse(*self.info.scores.get(pid).unwrap_or(&0)));
        players
    }
    pub fn new(window_size: Size, info: ScoreBoardInfo) -> Self {
        Self {
            window_size,
            btn_submit_bid: Button::new_submit_bid_button(21, 110, 36),
            info,
            showing_trump_select: false,
            hover_animation_red: HoverAnimation::new(20),
            hover_animation_green: HoverAnimation::new(20),
            hover_animation_blue: HoverAnimation::new(20),
            hover_animation_yellow: HoverAnimation::new(20),
        }
    }

    fn scoreboard_row<'a>(
        &self,
        name: &str,
        score: &str,
        tricks: &str,
        bid: &str,
        is_header: bool,
        is_current_turn: bool,
    ) -> Container<'a, AppMessage> {
        let text_color = if is_current_turn {
            Color::from_rgb(1.0, 0.85, 0.4)
        } else {
            Color::WHITE
        };

        let cell = |content: String| {
            container(
                text(content)
                    .size(if is_header {
                        self.size_small()
                    } else {
                        self.size_middle()
                    })
                    .color(text_color),
            )
            .width(iced::Length::FillPortion(1))
            .padding(4)
        };

        let name_cell = |content: String| {
            container(
                text(content)
                    .size(if is_header {
                        self.size_small()
                    } else {
                        self.size_middle()
                    })
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
                background: Some(if is_current_turn {
                    Color::from_rgba(1.0, 0.85, 0.4, 0.15).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, if is_header { 0.4 } else { 0.2 }).into()
                }),
                border: Border {
                    color: if is_current_turn {
                        Color::from_rgba(1.0, 0.85, 0.4, 0.5)
                    } else {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                    },
                    width: if is_current_turn { 1.5 } else { 1.0 },
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
    }

    fn reset_animations(&mut self) {
        self.hover_animation_blue.reset();
        self.hover_animation_green.reset();
        self.hover_animation_red.reset();
        self.hover_animation_yellow.reset();
    }
    fn not_show_trump_pannel(&self) -> bool {
        !self.info.must_set_trump || self.info.dealer != self.info.my_id
    }
    fn not_show_bidding_pannel(&self) -> bool {
        !self.info.is_bidding_phase || !self.info.is_my_turn || self.info.must_set_trump
    }
    fn build_bidding_panel<'a>(&self) -> Container<'a, AppMessage> {
        if self.not_show_bidding_pannel() {
            return container(None::<&str>);
        }
        let max_bid = self.info.round_number + 1;

        let is_valid_bid = if let Ok(bid) = self.info.bid_input.parse::<usize>() {
            if bid > max_bid {
                false
            } else {
                if max_bid != 1 {
                    // enforce sum != max_bid for the last bidder
                    let num_players = self.info.player_order.len();
                    let bids_placed = self.info.bids.len();
                    if bids_placed + 1 == num_players {
                        // last bidder
                        let sum_existing: usize = self.info.bids.values().sum();
                        sum_existing + bid != max_bid
                    } else {
                        true
                    }
                } else {
                    true // in the 1st round the sum can be 1
                }
            }
        } else {
            false
        };

        let bid_hint = if !self.info.bid_input.is_empty() {
            if let Ok(bid) = self.info.bid_input.parse::<usize>() {
                if bid > max_bid {
                    format!("Max bid is {max_bid}")
                } else if !is_valid_bid {
                    let sum_existing: usize = self.info.bids.values().sum();
                    let forbidden = max_bid - sum_existing;
                    format!("Can't bid {forbidden} as last bidder")
                } else {
                    format!("(0 to {max_bid})")
                }
            } else {
                "Enter a number".to_string()
            }
        } else {
            format!("(0 to {max_bid})")
        };

        let submit_button: iced::Element<'a, AppMessage> = if is_valid_bid {
            self.btn_submit_bid.view().into()
        } else {
            container(None::<&str>).into()
        };

        let panel = column![
            text("Bid:")
                .size(self.size_big())
                .color(Color::from_rgb(1.0, 1.0, 1.0)),
            row![
                text_input("Enter bid", &self.info.bid_input)
                    .on_input(AppMessage::BidInputChanged)
                    .width(self.bid_input_size()),
                self.btn_submit_bid.view(),
            ]
            .spacing(6),
            text(format!("(0 to {max_bid})"))
                .size(self.size_middle())
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
        ]
        .spacing(6);

        container(panel).padding([8, 0])
    }

    fn build_trump_panel<'a>(&self) -> Container<'a, AppMessage> {
        // Show only if dealer and must_set_trump
        if self.not_show_trump_pannel() {
            return container(None::<&str>);
        }
        let panel = column![
            text("Select Trump Suit:")
                .size(self.size_big())
                .color(Color::from_rgb(1.0, 1.0, 1.0)),
            row![
                mouse_area(
                    Image::new("assets/suits/red.png")
                        .filter_method(FilterMethod::Nearest)
                        .width(self.card_width())
                        .height(self.card_width() * CARD_WIDTH_HEIGHT_RATIO)
                        .scale(self.hover_animation_red.get_scale())
                )
                .interaction(Interaction::Pointer)
                .on_press(GameViewMessage::TryChooseSuit(Suit::Red).convert_msg())
                .on_enter(ScoreBoardMessage::SuitHovered(Suit::Red).convert_msg())
                .on_exit(ScoreBoardMessage::SuitNotHovered(Suit::Red).convert_msg()),
                mouse_area(
                    Image::new("assets/suits/green.png")
                        .filter_method(FilterMethod::Nearest)
                        .width(self.card_width())
                        .height(self.card_width() * CARD_WIDTH_HEIGHT_RATIO)
                        .scale(self.hover_animation_green.get_scale())
                )
                .interaction(Interaction::Pointer)
                .on_press(GameViewMessage::TryChooseSuit(Suit::Green).convert_msg())
                .on_enter(ScoreBoardMessage::SuitHovered(Suit::Green).convert_msg())
                .on_exit(ScoreBoardMessage::SuitNotHovered(Suit::Green).convert_msg()),
                mouse_area(
                    Image::new("assets/suits/blue.png")
                        .filter_method(FilterMethod::Nearest)
                        .width(self.card_width())
                        .height(self.card_width() * CARD_WIDTH_HEIGHT_RATIO)
                        .scale(self.hover_animation_blue.get_scale())
                )
                .interaction(Interaction::Pointer)
                .on_press(GameViewMessage::TryChooseSuit(Suit::Blue).convert_msg())
                .on_enter(ScoreBoardMessage::SuitHovered(Suit::Blue).convert_msg())
                .on_exit(ScoreBoardMessage::SuitNotHovered(Suit::Blue).convert_msg()),
                mouse_area(
                    Image::new("assets/suits/yellow.png")
                        .filter_method(FilterMethod::Nearest)
                        .width(self.card_width())
                        .height(self.card_width() * CARD_WIDTH_HEIGHT_RATIO)
                        .scale(self.hover_animation_yellow.get_scale())
                )
                .interaction(Interaction::Pointer)
                .on_press(GameViewMessage::TryChooseSuit(Suit::Yellow).convert_msg())
                .on_enter(ScoreBoardMessage::SuitHovered(Suit::Yellow).convert_msg())
                .on_exit(ScoreBoardMessage::SuitNotHovered(Suit::Yellow).convert_msg()),
            ]
            .spacing(6),
        ]
        .spacing(6);

        container(panel).padding([8, 0])
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

    fn size_small(&self) -> f32 {
        self.width() / 22.0
    }
    fn size_middle(&self) -> f32 {
        self.width() / 19.0
    }
    fn size_big(&self) -> f32 {
        self.width() / 16.0
    }
    fn size_huge(&self) -> f32 {
        self.width() / 10.0
    }
    fn card_width(&self) -> f32 {
        self.width() * 0.15
    }
    fn bid_input_size(&self) -> f32 {
        self.width() * 0.3
    }
}

impl Notifiable for ScoreBoard {
    type OwnMessage = ScoreBoardMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            ScoreBoardMessage::Update(info) => {
                self.info = info;
                if !self.not_show_trump_pannel() && self.showing_trump_select == false {
                    self.showing_trump_select = true;
                } else {
                    self.showing_trump_select = false;
                    self.reset_animations();
                }
            }
            ScoreBoardMessage::SuitHovered(suit) => match suit {
                Suit::Red => {
                    self.hover_animation_red.start();
                }
                Suit::Blue => {
                    self.hover_animation_blue.start();
                }
                Suit::Green => {
                    self.hover_animation_green.start();
                }
                Suit::Yellow => {
                    self.hover_animation_yellow.start();
                }
            },
            ScoreBoardMessage::SuitNotHovered(suit) => match suit {
                Suit::Red => {
                    self.hover_animation_red.reverse();
                }
                Suit::Blue => {
                    self.hover_animation_blue.reverse();
                }
                Suit::Green => {
                    self.hover_animation_green.reverse();
                }
                Suit::Yellow => {
                    self.hover_animation_yellow.reverse();
                }
            },
        }
        Task::none()
    }
}

impl ResizableDynHeight for ScoreBoard {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size
    }
    fn width(&self) -> f32 {
        SCOREBOARD_WIDTH_MUTL_WITH_WINDOW_WIDTH * self.window_size.width
    }
}

impl Animated for ScoreBoard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.btn_submit_bid.update_animations(),
            self.hover_animation_blue.next_frame(),
            self.hover_animation_green.next_frame(),
            self.hover_animation_red.next_frame(),
            self.hover_animation_yellow.next_frame(),
        ])
    }
}

impl Viewable for ScoreBoard {
    // AI Usage: overwrite the view so that the scoreboard is placed correctly
    // and uses rows+cells instead of rows+format strings
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut scores_col = Column::new().spacing(2);

        scores_col = scores_col.push(
            container(
                text("Scoreboard")
                    .size(self.size_huge())
                    .color(Color::WHITE),
            )
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
        scores_col = scores_col.push(self.scoreboard_row("Name", "Pkt", "Won", "Bid", true, false));

        for player_id in self.sorted_player_order_by_score() {
            let mut player_name = self.get_player_name(player_id);
            let score = self.info.scores.get(&player_id).unwrap_or(&0);
            let tricks = self.info.tricks_won.get(&player_id).unwrap_or(&0);
            let bid = self.info.bids.get(&player_id).unwrap_or(&0);
            let is_current_turn = self.info.current_player == Some(player_id);

            if player_name.is_empty() {
                player_name = "???".to_string();
            }

            scores_col = scores_col.push(self.scoreboard_row(
                &player_name,
                &score.to_string(),
                &tricks.to_string(),
                &bid.to_string(),
                false,
                is_current_turn,
            ));
        }

        scores_col = scores_col.push(
            container(
                text("Bids for current round")
                    .size(self.size_small())
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            )
            .padding([8, 0]),
        );

        if !self.not_show_trump_pannel() {
            scores_col = scores_col
                .push(self.build_trump_panel())
                .align_x(iced::Center);
        } else if !self.info.must_set_trump {
            scores_col = scores_col
                .push(self.build_bidding_panel())
                .align_x(iced::Center);
        }

        // Wrap in a styled container
        container(scores_col)
            .width(self.width())
            .height(Shrink)
            .padding([24.0, 24.0])
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
