use std::ops::Not;
use iced::{ContentFit::Fill, Point, Size, mouse::Interaction,
           widget::{Container, MouseArea, container, image, pin, stack}};
use crate::client::AppMessage;
use super::{animation_draw::DrawAnimation,
            animation_hover::HoverAnimation,
            animation_hover_focus::HoverFocusAnimation,
            animtion_hide::HideAnimation,
            animation_play::PlayAnimation,
            animation_playable::PlayableAnimation,
            animation_false_played::FalsePlayedAnimation,
            {f32_min_2, f32_min_3}
           };
use super::super::hand::{Hand, HandMessage};
use crate::ui_element_traits::*;
use crate::animation::animation::*;

static FRAME_PLAYABLE_PATH:&'static str = "assets/cards/frame_green.png";     
static FRAME_PLAYABLE_FOCUSED_PATH:&'static str = "assets/cards/frame_yellow.png";
static FALSE_PLAYED_PATH:&'static str = "assets/cards/false_played.png";

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
    ShowPlayableStatus(usize, bool)
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: usize,
    img_path: &'static str,
    pub size: Size,
    pub playable: bool,
    pub show_playable_status: bool,
    pub rotation: f32,
    pub draw_animation: DrawAnimation,
    pub hover_animation: HoverAnimation,
    pub play_animation: PlayAnimation,
    pub playable_animation: PlayableAnimation,
    pub false_played_animation: FalsePlayedAnimation,
    pub focus_animation: HoverFocusAnimation,
    pub hide_animation: HideAnimation
}

impl Card {

    pub fn new(id: usize, img_path: &'static str, size: Size, playable: bool) -> Self {
        let mut card: Card = Self {
            id: id,
            img_path: img_path,
            size: size,
            playable: playable,
            show_playable_status: false,
            rotation: 0.0,
            draw_animation: DrawAnimation::new(),
            hover_animation: HoverAnimation::new(size),
            play_animation: PlayAnimation::new(),
            playable_animation: PlayableAnimation::new(),
            false_played_animation: FalsePlayedAnimation::new(),
            focus_animation: HoverFocusAnimation::new(),
            hide_animation: HideAnimation::new()
        };
        card.playable_animation.start();
        card
    }
}

impl Message for Card {

    type OwnMessage = CardMessage;

    fn convert_to_app_message(msg: CardMessage) -> AppMessage {
        Hand::convert_to_app_message(HandMessage::CardMessage(msg))
    }

    fn update_with_msg(&mut self, msg: CardMessage) {
        match msg {
            CardMessage::Hovered(id) => {
                if id == self.id {
                    self.hover_animation.start();
                    self.focus_animation.start();
                }
            }
            CardMessage::Played(id) => {
                if id == self.id {
                    self.play_animation.start();
                }
            }
            CardMessage::NotHovered(id) => {
                if id == self.id {
                    self.hover_animation.reverse();
                    self.focus_animation.reset();
                    self.rotation = 0.0;
                }
            }
            CardMessage::Hide(id) => {
                if id == self.id {
                    self.hide_animation.start();
                }
            }
            CardMessage::Show(id) => {
                if id == self.id {
                    self.hide_animation.start_from_reverse();
                }
            }
            CardMessage::Draw(id) => {
                if id == self.id {
                    self.draw_animation.start();
                }
            }
            CardMessage::CursorMoved(id, point) => {
                if id == self.id {
                    let halve_card_width: f32 = self.size.width * (1.0/2.0);
                    let factor_width: f32 = (point.x - halve_card_width) / halve_card_width;

                    self.rotation = 0.05 * factor_width;
                }
            }
            CardMessage::FalsePlayed(id) => {
                if id == self.id {
                    self.false_played_animation.start();
                }
            }
            CardMessage::ShowPlayableStatus(id, do_show) => {
                if id == self.id {
                    self.show_playable_status = do_show;
                }
            }
        }
    }
}

impl Animated for Card {
    fn update_animations(&mut self) {
        self.draw_animation.next_frame();
        self.hover_animation.next_frame();
        self.play_animation.next_frame();
        self.playable_animation.next_frame();
        self.false_played_animation.next_frame();
        self.focus_animation.next_frame();
        self.hide_animation.next_frame();
    }
}

impl Resizable for Card {
    fn update_size(&mut self, window_size: Size) {
        self.size = window_size;
        self.hover_animation.update_target_max_offset(self.size);
    }
}

impl Viewable for Card {

    fn view<'a>(&self) -> Container<'a, AppMessage> {

        let img_opacity: f32 = f32_min_3(self.play_animation.get_opacity(),
                                         self.hide_animation.get_opacity(),
                                         self.draw_animation.get_opacity());

        let hover_effect_opacity: f32 = f32_min_2(img_opacity,
                                                  self.focus_animation.get_opacity());

        let playable_opacity: f32 = f32_min_2(img_opacity,
                                              self.playable_animation.get_opacity());

        let false_played_opacity: f32 = f32_min_2(img_opacity,
                                               self.false_played_animation.get_opacity());

        let width: f32 = self.size.width *
                         self.hover_animation.get_expansion() *
                         self.play_animation.get_contraction() *
                         self.hide_animation.get_contraction() *
                         self.draw_animation.get_contraction();

        let height: f32 = self.size.height *
                          self.hover_animation.get_expansion();

        let rotation: f32 = self.rotation;

        let scale: f32 = 0.88 * self.hide_animation.get_scale() * self.draw_animation.get_scale();


        let mut card = stack!();
        let img = image(self.img_path)
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
            let not_playable_effect = image(FALSE_PLAYED_PATH)
                .content_fit(Fill)
                .width(width)
                .height(height)
                .rotation(rotation)
                .scale(scale)
                .opacity(false_played_opacity);
            card = card.push(not_playable_effect)
        }

        let card_id = self.id.clone();
        let mut mouse_area = 
            MouseArea::new(card)
            .on_enter(Card::convert_to_app_message(CardMessage::Hovered(self.id)))
            .on_exit(Card::convert_to_app_message(CardMessage::NotHovered(self.id)))
            .on_right_press(Hand::convert_to_app_message(HandMessage::ShowPlayableStatus(self.show_playable_status.not())))
            .on_move(move |position| Card::convert_to_app_message(CardMessage::CursorMoved(card_id, position)))
            .interaction(Interaction::Pointer);
        if self.playable {
            mouse_area =
                mouse_area.on_double_click(
                    Card::convert_to_app_message(CardMessage::Played(self.id))
                )
        } else {
            mouse_area =
                mouse_area.on_double_click(
                    Card::convert_to_app_message(CardMessage::FalsePlayed(self.id))
                )
        }
        container(mouse_area)
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        
        container(pin(self.view())
            .position(
                Point::new(
                    x + ((1.0 - self.play_animation.get_contraction()) / 2.0) *
                           self.size.width * self.hover_animation.get_expansion() +
                           (self.size.width - self.size.width *
                            self.hover_animation.get_expansion()) /
                           2.0,
                    y - self.hover_animation.get_offset()
                )
            )
        )
    }
}

