mod animation_draw;
mod animation_false_played;
mod animation_focus;
mod animation_hide;
mod animation_hover;
mod animation_play;
mod animation_playable;

use crate::api::{
    get_card_path, Card, FALSE_PLAYED_PATH, FRAME_PLAYABLE_FOCUSED_PATH, FRAME_PLAYABLE_PATH,
};
use crate::client::{AppMessage, TaskBatcher};
use crate::gameplay_ui::hand::hand_card::{
    animation_draw::DrawAnimation, animation_false_played::FalsePlayedAnimation,
    animation_focus::FocusAnimation, animation_hide::HideAnimation,
    animation_hover::HoverAnimation, animation_play::PlayAnimation,
    animation_playable::PlayableAnimation,
};
use crate::gameplay_ui::hand::HandMessage;
use crate::gameplay_ui::table::middle::card_stack::CardStackMessage;
use crate::gameplay_ui::{card_heigth_hand, card_img_base_scale, card_width_hand};
use crate::ui_element_traits::*;
use iced::Task;
use iced::{
    mouse::Interaction,
    widget::{container, image, pin, stack, Container, MouseArea},
    ContentFit::Fill,
    Point, Size,
};
use std::ops::Not;

pub fn f32_min_2(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        return v1;
    }
    v2
}

#[derive(Debug, Clone)]
pub enum CardMessage {
    Played(usize),
    FalsePlayed(usize),
    Hovered(usize),
    NotHovered(usize),
    Hide(usize),
    Show(usize),
    Draw(usize),
    CursorMoved(usize, Point),
    ShowPlayableStatus(usize, bool),
    MakeClickable(usize),
}

impl Message for CardMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        HandMessage::convert_msg_from(HandMessage::CardMessage(msg))
    }
}

impl ReplaceUsize for CardMessage {
    fn replace_usize(&self, value: usize) -> Self {
        match self {
            CardMessage::Played(_) => CardMessage::Played(value),
            CardMessage::FalsePlayed(_) => CardMessage::FalsePlayed(value),
            CardMessage::Hovered(_) => CardMessage::Hovered(value),
            CardMessage::NotHovered(_) => CardMessage::NotHovered(value),
            CardMessage::Hide(_) => CardMessage::Hide(value),
            CardMessage::Show(_) => CardMessage::Show(value),
            CardMessage::Draw(_) => CardMessage::Draw(value),
            CardMessage::CursorMoved(_, point) => CardMessage::CursorMoved(value, *point),
            CardMessage::ShowPlayableStatus(_, bool) => {
                CardMessage::ShowPlayableStatus(value, *bool)
            }
            CardMessage::MakeClickable(_) => CardMessage::MakeClickable(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ViewableHandCard {
    id: usize,
    card: Card,
    img_path: String,
    window_size: Size,
    clickable: bool,
    playable: bool,
    show_playable_status: bool,
    rotation: f32,
    draw_animation: DrawAnimation,
    hover_animation: HoverAnimation,
    play_animation: PlayAnimation,
    playable_animation: PlayableAnimation,
    false_played_animation: FalsePlayedAnimation,
    focus_animation: FocusAnimation,
    hide_animation: HideAnimation,
}

impl ViewableHandCard {
    pub fn new(id: usize, card: Card, window_size: Size, playable: bool) -> Self {
        let play_duration: usize = 12;
        let mut viewable_card: ViewableHandCard = Self {
            id,
            card,
            img_path: get_card_path(card),
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
        };
        viewable_card
            .play_animation
            .on_special(HandMessage::DeleteCard(id).convert_msg());
        viewable_card
            .hide_animation
            .on_special(CardMessage::MakeClickable(id).convert_msg());
        viewable_card.playable_animation.start();
        viewable_card
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl Notifiable for ViewableHandCard {
    type OwnMessage = CardMessage;

    fn update_with_msg(&mut self, msg: CardMessage) -> Task<AppMessage> {
        let mut tb = TaskBatcher::new();
        match msg {
            CardMessage::Hovered(id) => {
                if id == self.id {
                    self.hover_animation.start();
                    self.focus_animation.start();
                } else {
                    // Sometimes on_exit for a viewed card won't register
                    // and won't send the CardNotHovered msg.
                    // To ensure that an unhovered card is not sticking up all the time
                    // send NotHovered to all cards except the hovered one.
                    tb.push(CardMessage::NotHovered(self.id).convert_msg_to_task())
                };
            }
            CardMessage::Played(id) => {
                if id == self.id {
                    self.clickable = false;
                    self.play_animation.start();
                    tb.push(CardStackMessage::CardPlayed(self.card.clone()).convert_msg_to_task());
                };
            }
            CardMessage::NotHovered(id) => {
                if id == self.id {
                    self.hover_animation.reverse();
                    self.focus_animation.reset();
                    self.rotation = 0.0;
                };
            }
            CardMessage::Hide(id) => {
                if id == self.id {
                    self.clickable = false;
                    self.hide_animation.start();
                };
            }
            CardMessage::Show(id) => {
                if id == self.id {
                    self.hide_animation.start_from_reverse();
                };
            }
            CardMessage::Draw(id) => {
                if id == self.id {
                    self.draw_animation.start();
                };
            }
            CardMessage::CursorMoved(id, point) => {
                if id == self.id {
                    let halve_card_width: f32 = self.width() / 2.0;
                    let factor_width: f32 = (point.x - halve_card_width) / halve_card_width;
                    // The factor 0.05 is representing a 5% rotation offset on maximum.
                    self.rotation = 0.05 * factor_width;
                };
            }
            CardMessage::FalsePlayed(id) => {
                if id == self.id {
                    self.false_played_animation.start();
                };
            }
            CardMessage::ShowPlayableStatus(id, do_show) => {
                if id == self.id {
                    self.show_playable_status = do_show;
                };
            }
            CardMessage::MakeClickable(id) => {
                if id == self.id {
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
        card_heigth_hand(self.window_size)
    }
}

impl SizeFromOutside for ViewableHandCard {
    fn width_for(window_size: Size) -> f32 {
        card_width_hand(window_size)
    }
    fn height_for(window_size: Size) -> f32 {
        card_heigth_hand(window_size)
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
        let img_opacity: f32 = f32_min_2(
            self.play_animation.get_opacity(),
            self.hide_animation.get_opacity(),
        );

        let hover_effect_opacity: f32 = f32_min_2(img_opacity, self.focus_animation.get_opacity());

        let playable_opacity: f32 = f32_min_2(img_opacity, self.playable_animation.get_opacity());

        let false_played_opacity: f32 =
            f32_min_2(img_opacity, self.false_played_animation.get_opacity());

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
        let img = image(self.img_path.clone())
            .content_fit(Fill)
            .width(width)
            .height(height)
            .rotation(rotation)
            .scale(scale)
            .opacity(img_opacity);
        card = card.push(img);
        if self.playable {
            if self.show_playable_status {
                let playable_effect = image(FRAME_PLAYABLE_PATH)
                    .content_fit(Fill)
                    .width(width)
                    .height(height)
                    .rotation(rotation)
                    .scale(scale)
                    .opacity(playable_opacity);
                card = card.push(playable_effect);
            }
            let hover_effect = image(FRAME_PLAYABLE_FOCUSED_PATH)
                .content_fit(Fill)
                .width(width)
                .height(height)
                .rotation(rotation)
                .scale(scale)
                .opacity(hover_effect_opacity);
            card = card.push(hover_effect);
        } else {
            let false_played_effect = image(FALSE_PLAYED_PATH)
                .content_fit(Fill)
                .width(width)
                .height(height)
                .rotation(rotation)
                .scale(scale)
                .opacity(false_played_opacity);
            card = card.push(false_played_effect)
        }

        let msg_hovered: AppMessage = CardMessage::Hovered(self.id).convert_msg();
        let msg_not_hoverd: AppMessage = CardMessage::NotHovered(self.id).convert_msg();
        let msg_show_playable_status =
            HandMessage::ShowPlayableStatus(self.show_playable_status.not()).convert_msg();
        let card_id: usize = self.id;
        let msg_cursor_moved =
            move |position: Point| CardMessage::CursorMoved(card_id, position).convert_msg();
        let msg_played = CardMessage::Played(self.id).convert_msg();
        let msg_false_played = CardMessage::FalsePlayed(self.id).convert_msg();

        let mut mouse_area = MouseArea::new(card)
            .on_enter(msg_hovered)
            .on_exit(msg_not_hoverd)
            .on_right_press(msg_show_playable_status)
            .on_move(msg_cursor_moved)
            .interaction(Interaction::Pointer);
        if self.playable {
            mouse_area = mouse_area.on_double_click(msg_played)
        } else {
            mouse_area = mouse_area.on_double_click(msg_false_played)
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
