mod animation_draw;
mod animation_false_played;
mod animation_focus;
mod animation_hide;
mod animation_hover;
mod animation_play;
mod animation_playable;

use crate::api::{Card, FALSE_PLAYED_PATH, FRAME_PLAYABLE_FOCUSED_PATH, FRAME_PLAYABLE_PATH};
use crate::client::{AppMessage, TaskBatcher};
use crate::gameplay_ui::hand::hand_card::{
    animation_draw::DrawAnimation, animation_false_played::FalsePlayedAnimation,
    animation_focus::FocusAnimation, animation_hide::HideAnimation,
    animation_hover::HoverAnimation, animation_play::PlayAnimation,
    animation_playable::PlayableAnimation,
};
use crate::gameplay_ui::hand::HandMessage;
use crate::gameplay_ui::{card_height_hand, card_img_base_scale, card_width_hand, GameViewMessage};
use crate::ui_element_traits::*;
use iced::Task;
use iced::{
    mouse::Interaction,
    widget::{container, image, pin, stack, Container, MouseArea},
    ContentFit::Fill,
    Point, Size,
};
use std::ops::Not;

#[derive(Debug, Clone)]
pub enum CardMessage {
    Played(Card),
    Clicked(Card),
    Hovered(Card),
    NotHovered(Card),
    Hide(Card),
    Show(Card),
    CursorMoved(Card, Point),
    ShowPlayableStatus(Card, bool),
    MakeClickable(Card),
}

impl Message for CardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        HandMessage::convert_msg_from(HandMessage::CardMessage(msg))
    }
}

#[derive(Debug, Clone)]
pub struct ViewableHandCard {
    pub my_turn: bool,
    pub card: Card,
    window_size: Size,
    clickable: bool,
    playable: bool,
    show_playable_status: bool,
    rotation: f32,

    pub draw_animation: DrawAnimation,
    hover_animation: HoverAnimation,
    play_animation: PlayAnimation,
    playable_animation: PlayableAnimation,
    false_played_animation: FalsePlayedAnimation,
    focus_animation: FocusAnimation,
    hide_animation: HideAnimation,

    img_handle: iced::widget::image::Handle,
    img_frame_playable: iced::widget::image::Handle,
    img_frame_playable_focused: iced::widget::image::Handle,
    img_false_played: iced::widget::image::Handle,
}

impl ViewableHandCard {
    pub fn new(card: Card, window_size: Size, playable: bool) -> Self {
        let play_duration: usize = 12;
        let mut viewable_card: ViewableHandCard = Self {
            my_turn: false,
            card,
            window_size,
            clickable: true,
            playable,
            show_playable_status: false,
            rotation: 0.0,

            draw_animation: DrawAnimation::new(10),
            hover_animation: HoverAnimation::new(5),
            play_animation: PlayAnimation::new(play_duration),
            playable_animation: PlayableAnimation::new(100),
            false_played_animation: FalsePlayedAnimation::new(25),
            focus_animation: FocusAnimation::new(70),
            hide_animation: HideAnimation::new(play_duration),

            img_handle: iced::widget::image::Handle::from_path(card.img_path()),
            img_frame_playable: iced::widget::image::Handle::from_path(FRAME_PLAYABLE_PATH),
            img_frame_playable_focused: iced::widget::image::Handle::from_path(
                FRAME_PLAYABLE_FOCUSED_PATH,
            ),
            img_false_played: iced::widget::image::Handle::from_path(FALSE_PLAYED_PATH),
        };
        viewable_card
            .play_animation
            .on_end_reached(HandMessage::DeleteCard(card).convert_msg());
        viewable_card
            .hide_animation
            .on_end_reached(CardMessage::MakeClickable(card).convert_msg());
        viewable_card.playable_animation.start_infinite();
        viewable_card
    }
    pub fn validate(&mut self, valid_cards: Vec<Card>) {
        if valid_cards.contains(&self.card) {
            self.playable = true;
        } else {
            self.playable = false;
        }
    }
}

impl Notifiable for ViewableHandCard {
    type OwnMessage = CardMessage;

    fn update_with_msg(&mut self, msg: CardMessage) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        match msg {
            CardMessage::Hovered(card) => {
                if card == self.card {
                    println!("hovered");
                    tb.push(self.hover_animation.start());
                    tb.push(self.focus_animation.start());
                } else {
                    // Sometimes on_exit for a viewed card won't register
                    // and won't send the CardNotHovered msg.
                    // To ensure that an unhovered card is not sticking up all the time
                    // send NotHovered to all cards except the hovered one.
                    tb.push(CardMessage::NotHovered(card).convert_msg_to_task())
                };
            }
            CardMessage::Played(card) => {
                if card == self.card {
                    if self.playable {
                        self.clickable = false;
                        tb.push(self.play_animation.start());
                    } else {
                        tb.push(self.false_played_animation.start());
                    }
                };
            }
            CardMessage::Clicked(card) => {
                if card == self.card {
                    if self.playable {
                        tb.push(GameViewMessage::TryPlayCard(self.card).convert_msg_to_task())
                    } else {
                        tb.push(self.false_played_animation.start());
                    }
                }
            }
            CardMessage::NotHovered(card) => {
                if card == self.card {
                    tb.push(self.hover_animation.reverse());
                    tb.push(self.focus_animation.reset());
                    self.rotation = 0.0;
                };
            }
            CardMessage::Hide(card) => {
                if card == self.card {
                    self.clickable = false;
                    tb.push(self.hide_animation.start());
                };
            }
            CardMessage::Show(card) => {
                if card == self.card {
                    tb.push(self.hide_animation.reverse());
                };
            }
            CardMessage::CursorMoved(card, point) => {
                if card == self.card {
                    let halve_card_width: f32 = self.width() / 2.0;
                    let factor_width: f32 = (point.x - halve_card_width) / halve_card_width;
                    // The factor 0.05 is representing a 5% rotation offset on maximum.
                    self.rotation = 0.05 * factor_width;
                };
            }
            CardMessage::ShowPlayableStatus(card, do_show) => {
                if card == self.card {
                    self.show_playable_status = do_show;
                };
            }
            CardMessage::MakeClickable(card) => {
                if card == self.card {
                    self.clickable = true;
                }
            }
        }
        tb.batch()
    }
}

impl Animated for ViewableHandCard {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.draw_animation.next_frame(),
            self.hover_animation.next_frame(),
            self.play_animation.next_frame(),
            self.playable_animation.next_frame(),
            self.false_played_animation.next_frame(),
            self.focus_animation.next_frame(),
            self.hide_animation.next_frame(),
        ])
    }
}

impl Resizable for ViewableHandCard {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
    }
    fn width(&self) -> f32 {
        card_width_hand(self.window_size)
    }
    fn height(&self) -> f32 {
        card_height_hand(self.window_size)
    }
}

impl SizeFromOutside for ViewableHandCard {
    fn width_for(window_size: Size) -> f32 {
        card_width_hand(window_size)
    }
    fn height_for(window_size: Size) -> f32 {
        card_height_hand(window_size)
    }
}

impl Viewable for ViewableHandCard {
    /// DON'T USE THIS!!!
    ///
    /// Instead use view_and_move for a card with x & y at 0.0,
    /// because view alone does not calculate the correct offset.
    /// This calculation is moved to view_and_move, because otherwise
    /// some effects of the card won't render for an unknown reason.
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        let img_opacity: f32 = self
            .play_animation
            .get_opacity()
            .min(self.hide_animation.get_opacity());

        let width: f32 = self.width()
            * self.hover_animation.get_expansion()
            * self.play_animation.get_contraction()
            * self.hide_animation.get_contraction()
            * self.draw_animation.get_contraction();

        let height: f32 = self.height() * self.hover_animation.get_expansion();

        let rotation: f32 = self.rotation;

        let scale: f32 = card_img_base_scale()
            * self.hide_animation.get_scale()
            * self.draw_animation.get_scale();

        let mut card = stack!();
        let img = image(self.img_handle.clone())
            .content_fit(Fill)
            .width(width)
            .height(height)
            .rotation(rotation)
            .scale(scale)
            .opacity(img_opacity);
        card = card.push(img);

        if self.my_turn {
            let hover_effect_opacity: f32 = self.focus_animation.get_opacity().min(img_opacity);
            let playable_opacity: f32 = self.playable_animation.get_opacity().min(img_opacity);
            let false_played_opacity: f32 =
                self.false_played_animation.get_opacity().min(img_opacity);
            if self.playable {
                if self.show_playable_status {
                    let playable_effect = image(self.img_frame_playable.clone())
                        .content_fit(Fill)
                        .width(width)
                        .height(height)
                        .rotation(rotation)
                        .scale(scale)
                        .opacity(playable_opacity);
                    card = card.push(playable_effect);
                }
                let hover_effect = image(self.img_frame_playable_focused.clone())
                    .content_fit(Fill)
                    .width(width)
                    .height(height)
                    .rotation(rotation)
                    .scale(scale)
                    .opacity(hover_effect_opacity);
                card = card.push(hover_effect);
            } else {
                let false_played_effect = image(self.img_false_played.clone())
                    .content_fit(Fill)
                    .width(width)
                    .height(height)
                    .rotation(rotation)
                    .scale(scale)
                    .opacity(false_played_opacity);
                card = card.push(false_played_effect)
            }
        };

        let msg_hovered: AppMessage = CardMessage::Hovered(self.card).convert_msg();
        let msg_not_hoverd: AppMessage = CardMessage::NotHovered(self.card).convert_msg();
        let card_data = self.card;
        let msg_cursor_moved =
            move |position: Point| CardMessage::CursorMoved(card_data, position).convert_msg();

        let mut mouse_area = MouseArea::new(card)
            .on_enter(msg_hovered)
            .on_exit(msg_not_hoverd)
            .on_move(msg_cursor_moved);

        if self.my_turn {
            let msg_show_playable_status =
                HandMessage::ShowPlayableStatus(self.show_playable_status.not()).convert_msg();
            let msg_double_clicked = CardMessage::Clicked(self.card).convert_msg();
            let interaction: Interaction = if self.playable {
                Interaction::Pointer
            } else {
                Interaction::NotAllowed
            };
            mouse_area = mouse_area
                .on_right_press(msg_show_playable_status)
                .on_double_click(msg_double_clicked)
                .interaction(interaction);
        }

        container(mouse_area)
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        // Construct the offset of the card before moving it.
        // x: repositioning the card back to the middle after beeing moved around by animations.
        // Note: This can be ingored on the y-axis (because the card moving up with the
        //       hover animation cancels the offset out).
        let x_offset: f32 = ((1.0 - self.play_animation.get_contraction()) / 2.0)
            * self.width()
            * self.hover_animation.get_expansion()
            + (self.width() - self.width() * self.hover_animation.get_expansion()) / 2.0;
        // y: realize the card moving up animation.
        let y_offset: f32 = -self.hover_animation.get_offset(self.window_size);
        let corrected_position: Point = Point::new(x + x_offset, y + y_offset);

        container(pin(self.view()).position(corrected_position))
    }
}
