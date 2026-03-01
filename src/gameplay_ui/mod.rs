#![allow(dead_code)]
pub mod hand;
pub mod scoreboard;
pub mod table;

use derive_more::{Deref, DerefMut};
use iced::{
    ContentFit,
    Length::Fill,
    Point, Size, Task,
    widget::{Container, container, image, mouse_area, pin, stack},
};

use crate::{
    animation::{BasicAnimation, Easing},
    api::{Card, Player, PlayerId, Suit},
    client::{App, AppMessage, TaskBatcher, audio::Sfx, views::ButtonMessage},
    gameplay_ui::{
        hand::{HandMessage, ViewableHand},
        scoreboard::{ScoreBoard, ScoreBoardInfo, ScoreBoardMessage},
        table::{
            TableMessage, ViewableTable,
            avatar::AvatarMessage,
            middle::{card_deck::CardDeckMessage, card_stack::CardStackMessage},
        },
    },
    ui_element_traits::{Animated, Message, Notifiable, Resizable, ResizableDynHeight, Viewable},
};

static TABLE_Y_POSITION_MULT_WITH_WINDOW_HEIGHT: f32 = 0.1;

static SCOREBOARD_WIDTH_MUTL_WITH_WINDOW_WIDTH: f32 = 0.2;

static AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH: f32 = 0.1;
static AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH: f32 = AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH * 0.75;
static AVATAR_SHARD_SIZE_MULT_WITH_WINDOW_WIDTH: f32 =
    (AVATAR_SIZE_MULT_WITH_WINDOW_WIDTH - AVATAR_IMG_SIZE_MULT_WITH_WINDOW_WIDTH) / 2.0;
static AVATAR_FRAME_WIDTH_HEIGHT_RATIO: f32 = 1.2;

// The hand size is depending on the window size with the factor 0.1.
static CARD_WIDTH_MULT_WITH_WINDOW_WIDTH: f32 = 0.1;
// 1.54 is around 1245 / 806 (height to width ratio of a card image).
static CARD_WIDTH_HEIGHT_RATIO: f32 = 1.54;
pub static CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH: f32 = CARD_WIDTH_MULT_WITH_WINDOW_WIDTH * 1.54;

// Adjust this arbitrary value to manipulate the width of the hand relative to its size,
// but be careful that the cards don't go out of screen.
// If you want to manipulate the total hand size
// change the value of hand_card::CARD_WIDTH_MULT_WITH_WINDOW_WIDTH.
static CARD_COLUMN_STEP_MULT_WITH_CARD_WIDTH: f32 = 1.0 / 3.0;
// Adjust this arbitrary value to manipulate the height of the hand,
static CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH: f32 = CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH * 0.43;

// The factor is chosen so the card img will not get clipped when rotated.
static CARD_IMG_BASE_SCALE: f32 = 0.92;
static CARD_IMG_MIDDLE_BASE_SCALE: f32 = 0.8;

static CARD_AREA_MIDDLE_RELATION: f32 = 1.3;

fn card_width_hand(window_size: Size) -> f32 {
    CARD_WIDTH_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_height_hand(window_size: Size) -> f32 {
    CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_column_step_hand(card_width: f32) -> f32 {
    CARD_COLUMN_STEP_MULT_WITH_CARD_WIDTH * card_width
}
fn card_row_step_hand(window_size: Size) -> f32 {
    CARD_ROW_STEP_MULT_WITH_WINDOW_WIDTH * window_size.width
}
fn card_img_base_scale() -> f32 {
    CARD_IMG_BASE_SCALE
}
fn card_img_middle_base_scale() -> f32 {
    CARD_IMG_MIDDLE_BASE_SCALE
}
fn card_width_middle(window_size: Size) -> f32 {
    card_width_hand(window_size) * (card_img_base_scale() / card_img_middle_base_scale())
}
fn card_height_middle(window_size: Size) -> f32 {
    card_height_hand(window_size) * (card_img_base_scale() / card_img_middle_base_scale())
}
fn card_area_middle_space_width(window_size: Size) -> f32 {
    card_width_middle(window_size) * CARD_AREA_MIDDLE_RELATION
}
fn card_area_middle_space_height(window_size: Size) -> f32 {
    card_height_middle(window_size) * CARD_AREA_MIDDLE_RELATION
}
fn card_area_middle_spawn_point(width: f32, height: f32, window_size: Size) -> Point {
    Point::new(
        (card_area_middle_space_width(window_size) - width) / 2.0,
        (card_area_middle_space_height(window_size) - height) / 2.0,
    )
}

#[derive(Clone, Debug)]
pub struct GameStartInfo {
    players: Vec<Player>,
    my_id: PlayerId,
    sb_info: ScoreBoardInfo,
}

impl GameStartInfo {
    pub fn new(app: &App) -> Self {
        Self {
            players: app.lobby.as_ref().unwrap().players.clone(),
            my_id: *app.my_id.as_ref().unwrap(),
            sb_info: app.scoreboard_info(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GameViewMessage {
    // gui -> gui
    HandMessage(HandMessage),
    TableMessage(TableMessage),
    ScoreBoardMessage(ScoreBoardMessage),

    // server -> gui
    StartGame(Box<GameStartInfo>),
    EndGame(PlayerId),
    NewRound(Option<Card>, Vec<Card>, Vec<Card>),
    NewTrick,
    ChangeTurn(PlayerId, Vec<Card>),
    CardPlayed(PlayerId, Card),

    // gui -> server
    TryPlayCard(Card),
    TryBid,
    TryChooseSuit(Suit),
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct BlackScreenFadeInAnimation(BasicAnimation);

impl BlackScreenFadeInAnimation {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration))
    }
    pub fn get_opacity(&self) -> f32 {
        self.progress(Easing::InSine) * 0.8
    }
}

impl Message for GameViewMessage {
    fn convert_msg_from(msg: Self) -> crate::client::AppMessage {
        AppMessage::GameViewMessage(Box::new(msg))
    }
}

pub struct GameView {
    window_size: Size,
    img_ingame_background: image::Handle,

    // game data
    my_id: Option<PlayerId>,
    game_ended: bool,
    game_ended_animation: BlackScreenFadeInAnimation,

    // gui elements
    viewable_hand: ViewableHand,
    viewable_table: ViewableTable,
    scoreboard: ScoreBoard,
}

impl GameView {
    pub fn new(window_size: Size) -> Self {
        Self {
            window_size,
            img_ingame_background: image::Handle::from_path("assets/ingame_background.png"),
            my_id: None,
            game_ended: false,
            game_ended_animation: BlackScreenFadeInAnimation::new(150),
            viewable_hand: ViewableHand::new(window_size),
            viewable_table: ViewableTable::new(window_size),
            scoreboard: ScoreBoard::new(window_size, ScoreBoardInfo::default()),
        }
    }
    pub fn update_buttons_with_msg(&mut self, btn_msg: ButtonMessage) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.scoreboard
                .btn_submit_bid
                .update_with_msg(btn_msg.clone()),
            self.scoreboard.btn_submit_bid.update_with_msg(btn_msg),
        ])
    }
}

impl Notifiable for GameView {
    type OwnMessage = GameViewMessage;
    fn update_with_msg(&mut self, msg: Self::OwnMessage) -> Task<AppMessage> {
        match msg {
            GameViewMessage::HandMessage(hand_msg) => self.viewable_hand.update_with_msg(hand_msg),
            GameViewMessage::TableMessage(table_msg) => {
                self.viewable_table.update_with_msg(table_msg)
            }
            GameViewMessage::ScoreBoardMessage(sb_msg) => self.scoreboard.update_with_msg(sb_msg),
            GameViewMessage::StartGame(info) => {
                let mut tb = TaskBatcher::new();
                self.my_id = Some(info.my_id);
                self.viewable_hand.my_id = Some(info.my_id);
                tb.push(
                    self.scoreboard
                        .update_with_msg(ScoreBoardMessage::Update(Box::new(info.sb_info))),
                );
                self.viewable_table.build_avatars(info.players);
                tb.batch()
            }
            GameViewMessage::CardPlayed(played_by, card) => {
                let mut tb = TaskBatcher::new();
                tb.push_msg(CardStackMessage::CardPlayed(card));
                tb.push_msg(AvatarMessage::PlayShard(played_by));
                if played_by == self.my_id.unwrap() {
                    tb.push_msg(HandMessage::PlayedCard(card))
                };
                tb.batch()
            }
            GameViewMessage::NewRound(trump_card, hand_cards, valid_cards) => {
                let mut tb = TaskBatcher::new();
                let hand_cards_len = hand_cards.len();
                tb.push_msg(HandMessage::DrawCards(hand_cards, valid_cards));
                tb.push_msg(CardStackMessage::HideAllCards);
                tb.push_msg(CardDeckMessage::Deal(hand_cards_len, trump_card));
                tb.push_msg(TableMessage::DrawShards(hand_cards_len));
                tb.push_msg(HandMessage::NobodiesTurn);
                tb.batch()
            }
            GameViewMessage::NewTrick => CardStackMessage::HideAllCards.convert_msg_to_task(),
            GameViewMessage::ChangeTurn(player_id, valid_cards) => TaskBatcher::instant_batch([
                TableMessage::ChangeTurn(player_id).convert_msg_to_task(),
                HandMessage::ChangeTurn(player_id, valid_cards).convert_msg_to_task(),
            ]),
            GameViewMessage::TryPlayCard(card) => AppMessage::PlayCard(card).convert_msg_to_task(),
            GameViewMessage::TryBid => AppMessage::SubmitBid.convert_msg_to_task(),
            GameViewMessage::TryChooseSuit(suit) => TaskBatcher::instant_batch([
                AppMessage::SetTrump(suit).convert_msg_to_task(),
                AppMessage::PlaySfx(Sfx::Click).convert_msg_to_task(),
            ]),
            GameViewMessage::EndGame(_) => {
                self.game_ended = true;
                self.game_ended_animation.start();
                self.scoreboard.move_end_board_animation.start();
                Task::done(AppMessage::PlaySfx(Sfx::GameOver))
            }
        }
    }
}

impl Animated for GameView {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.viewable_hand.update_animations(),
            self.viewable_table.update_animations(),
            self.scoreboard.update_animations(),
            self.game_ended_animation.next_frame(),
        ])
    }
}

impl Resizable for GameView {
    fn height(&self) -> f32 {
        self.window_size.height
    }
    fn width(&self) -> f32 {
        self.window_size.width
    }
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.viewable_hand.update_size(window_size);
        self.viewable_table.update_size(window_size);
        self.scoreboard.update_size(window_size);
    }
}

impl Viewable for GameView {
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let mut content = stack!().width(self.width()).height(self.height());
        // Background
        content =
            content.push(image(self.img_ingame_background.clone()).content_fit(ContentFit::Cover));
        // Scoreboard
        content = content.push(
            self.scoreboard
                .view_and_move(self.width() - self.scoreboard.width(), 0.0),
        );
        // Card Table
        content = content.push(self.viewable_table.view_and_move(
            (self.width() - self.viewable_table.width()) / 2.0,
            self.height() * TABLE_Y_POSITION_MULT_WITH_WINDOW_HEIGHT,
        ));
        // Card Hand
        content = content.push(self.viewable_hand.view_and_move(
            (self.width() - self.viewable_hand.width()) / 2.0,
            self.height() - self.viewable_hand.height(),
        ));
        if self.game_ended {
            let mut winner_avatar = self
                .viewable_table
                .find_avatar(self.my_id.unwrap())
                .unwrap();
            winner_avatar.turn_frame_animation.reset();
            winner_avatar.turn_frame_glow_animation.reset();
            winner_avatar.shards = 0;
            content = content.push(mouse_area(
                image("assets/black_screen.png")
                    .content_fit(ContentFit::Fill)
                    .opacity(self.game_ended_animation.get_opacity())
                    .width(Fill)
                    .height(Fill),
            ));
            content = content.push(
                pin(self.scoreboard.view_as_game_end_board(winner_avatar)).position(Point::new(
                    (self.width() - self.scoreboard.width()) * 0.5,
                    self.height() * 0.2
                        - self.scoreboard.move_end_board_animation.get_offset() * self.height(),
                )),
            );
        }
        container(content)
    }
}
