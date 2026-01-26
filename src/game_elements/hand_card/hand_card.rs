use iced::{ContentFit::Fill, Point, Size, mouse::Interaction, widget::{Container, MouseArea, container, image, pin, stack}};
use super::super::{GameElement, AnimationCore, ReversableAnimation, RepeatingAnimation, BasicAnimation};
use crate::game_elements::hand::{Hand, HandMessage};
use crate::client::AppMessage;
use super::{animation_draw::DrawAnimation,
            animation_hover::HoverAnimation,
            animation_hover_focus::HoverFocusAnimation,
            animtion_hide::HideAnimation,
            animation_play::PlayAnimation,
            {f32_min_2, f32_min_3}
           };

#[derive(Debug, Clone)]
pub enum CardMessage {
    Played(usize),
    Hovered(usize),
    NotHovered(usize),
    Hide(usize),
    Show(usize),
    Draw(usize),
    CursorMoved(usize, Point)
}

impl CardMessage {
    pub fn get_id(&self) -> usize {
        match self {
            CardMessage::Hovered(id) => *id,
            CardMessage::NotHovered(id) => *id,
            CardMessage::Played(id) => *id,
            CardMessage::Hide(id) => *id,
            CardMessage::Show(id) => *id,
            CardMessage::Draw(id) => *id,
            CardMessage::CursorMoved(id, _) => *id
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: usize,
    img_path: &'static str,
    pub size: Size,
    pub contraction_width: f32,
    pub contraction_height: f32,
    pub rotation: f32,
    pub draw_animation: DrawAnimation,
    pub hover_animation: HoverAnimation,
    pub play_animation: PlayAnimation,
    pub focus_animation: HoverFocusAnimation,
    pub hide_animation: HideAnimation
}

impl Card {

    pub fn new(id: usize, img_path: &'static str, size: Size) -> Self {
        Self {
            id: id,
            img_path: img_path,
            size: size,
            contraction_width: 1.0,
            contraction_height: 1.0,
            rotation: 0.0,
            draw_animation: DrawAnimation::new(),
            hover_animation: HoverAnimation::new(size),
            play_animation: PlayAnimation::new(),
            focus_animation: HoverFocusAnimation::new(),
            hide_animation: HideAnimation::new()
        }
    }
}

impl GameElement for Card {

    type OwnMessage = CardMessage;

    fn convert_to_app_message(msg: CardMessage) -> AppMessage {
        Hand::convert_to_app_message(HandMessage::CardMessage(msg))
    }

    fn update_with_msg(&mut self, msg: CardMessage) {
        if self.id == msg.get_id() {
            match msg {
                CardMessage::Hovered(_) => {
                    self.hover_animation.start();
                    self.focus_animation.start();
                }
                CardMessage::Played(_) => {
                    self.play_animation.start();
                }
                CardMessage::NotHovered(_) => {
                    self.hover_animation.reverse();
                    self.focus_animation.reset();
                    self.contraction_height = 1.0;
                    self.contraction_width = 1.0;
                    self.rotation = 0.0;
                }
                CardMessage::Hide(_) => {
                    self.hide_animation.start();
                }
                CardMessage::Show(_) => {
                    self.hide_animation.start_from_reverse();
                }
                CardMessage::Draw(_) => {
                    self.draw_animation.start();
                }
                CardMessage::CursorMoved(_, point) => {
                    let halve_card_width: f32 = self.size.width * (1.0/2.0);
                    let halve_card_height: f32 = self.size.height * (1.0/2.0);
                    let point_from_middle: Point = Point::new(point.x - halve_card_width,
                                                              point.y - halve_card_height);
                    let factor_width: f32 = point_from_middle.x / halve_card_width;
                    let factor_height: f32 = point_from_middle.y / halve_card_height;

                    self.contraction_height = 0.95 + 0.05 * (1.0 - factor_height.abs());
                    self.contraction_width = 0.95 + 0.05 * (1.0 - factor_width.abs());
                    self.rotation = -0.05 * factor_width.abs() * factor_height *
                                    (if factor_width > 0.0 {-1.0} else {1.0});
                }
            }
        }
    }

    fn update_animations(&mut self) {
        self.draw_animation.next_frame();
        self.hover_animation.next_frame();
        self.play_animation.next_frame();
        self.focus_animation.next_frame();
        self.hide_animation.next_frame();
    }

    fn update_size(&mut self, window_size: Size) {
        self.size = window_size;
        self.hover_animation.update_target_max_offset(self.size);
    }

    fn view<'a>(&self) -> Container<'a, AppMessage> {

        let img_opacity: f32 = f32_min_3(self.play_animation.get_opacity(),
                                         self.hide_animation.get_opacity(),
                                         self.draw_animation.get_opacity());

        let hover_effect_opacity = f32_min_2(img_opacity,
                                                  self.focus_animation.get_opacity());

        let width: f32 = self.size.width * self.contraction_width *
                         self.hover_animation.get_expansion() *
                         self.play_animation.get_contraction() *
                         self.hide_animation.get_contraction() *
                         self.draw_animation.get_contraction();

        let height: f32 = self.size.height * self.contraction_height *
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
        let hover_effect = image(self.focus_animation.img_path)
                    .content_fit(Fill)
                    .width(width)
                    .height(height)
                    .rotation(rotation)
                    .scale(scale)
                    .opacity(hover_effect_opacity);
        card = card.push(hover_effect);

        let card_id = self.id.clone();
        container(MouseArea::new(card)
            .on_double_click(Card::convert_to_app_message(CardMessage::Played(self.id)))
            .on_enter(Card::convert_to_app_message(CardMessage::Hovered(self.id)))
            .on_exit(Card::convert_to_app_message(CardMessage::NotHovered(self.id)))
            .on_move(move |position| Card::convert_to_app_message(CardMessage::CursorMoved(card_id, position)))
            .interaction(Interaction::Pointer)
        )
    }

    fn view_and_move<'a>(&self, x: f32, y: f32) -> Container<'a, AppMessage> {
        
        container(pin(self.view())
            .position(
                Point::new(
                    x + ((1.0 - self.play_animation.get_contraction()) / 2.0) *
                           self.size.width * self.hover_animation.get_expansion() +
                           (self.size.width - self.contraction_width * self.size.width) / 2.0 +
                           (self.size.width - self.size.width *
                            self.hover_animation.get_expansion()) /
                           2.0,
                    y - self.hover_animation.get_offset() + (self.size.height - self.contraction_height * self.size.height) / 2.0
                )
            )
        )
    }
}
