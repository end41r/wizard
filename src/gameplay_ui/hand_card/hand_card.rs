use std::ops::Not;
use iced::{ContentFit::Fill, Point, Size, mouse::Interaction,
           widget::{Container, MouseArea, container, image, pin, stack}};
use crate::client::AppMessage;
use super::{animation_draw::DrawAnimation,
            animation_hover::HoverAnimation,
            animation_hover_focus::HoverFocusAnimation,
            animation_hide::HideAnimation,
            animation_play::PlayAnimation,
            animation_playable::PlayableAnimation,
            animation_false_played::FalsePlayedAnimation,
            f32_min_2
           };
use super::super::hand::{ViewableHand, HandMessage};
use crate::ui_element_traits::*;
use crate::animation::animation::*;

static FRAME_PLAYABLE_PATH:&'static str = "assets/cards/frame_green.png";     
static FRAME_PLAYABLE_FOCUSED_PATH:&'static str = "assets/cards/frame_yellow.png";
static FALSE_PLAYED_PATH:&'static str = "assets/cards/false_played.png";

// The hand size is depending on the window size with the factor 0.1.
static CARD_WIDTH_MULT_WITH_WINDOW_WIDTH: f32 = 0.1;
// 1.54 is around 1245 / 806 (height to width ratio of a card image).
pub static CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH: f32 = CARD_WIDTH_MULT_WITH_WINDOW_WIDTH * 1.54;

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
pub struct ViewableCard {
    pub id: usize,
    img_path: &'static str,
    pub window_size: Size,
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

impl ViewableCard {

    pub fn new(id: usize, img_path: &'static str, window_size: Size, playable: bool) -> Self {
        let mut card: ViewableCard = Self {
            id: id,
            img_path: img_path,
            window_size: window_size,
            playable: playable,
            show_playable_status: false,
            rotation: 0.0,
            draw_animation: DrawAnimation::new(),
            hover_animation: HoverAnimation::new(ViewableCard::height_for(window_size)),
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

impl Message for ViewableCard {

    type OwnMessage = CardMessage;

    fn convert_to_app_message(msg: CardMessage) -> AppMessage {
        ViewableHand::convert_to_app_message(HandMessage::CardMessage(msg))
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
                    let halve_card_width: f32 = self.width() / 2.0;
                    let factor_width: f32 = (point.x - halve_card_width) / halve_card_width;
                    // The factor 0.05 is representing a 5% rotation offset on maximum.
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

impl Animated for ViewableCard {
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

impl Resizable for ViewableCard {
    fn update_size(&mut self, window_size: Size) {
        self.window_size = window_size;
        self.hover_animation.update_max_offset(ViewableCard::height_for(self.window_size));
    }
    fn width(&self) -> f32 {
        self.window_size.width * CARD_WIDTH_MULT_WITH_WINDOW_WIDTH
    }
    fn height(&self) -> f32 {
        self.window_size.width * CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH
    }
}

impl SizeFromOutside for ViewableCard {
    fn width_for(window_size: Size) -> f32 {
        window_size.width * CARD_WIDTH_MULT_WITH_WINDOW_WIDTH
    }
    fn height_for(window_size: Size) -> f32 {
        window_size.width * CARD_HEIGHT_MULT_WITH_WINDOW_WIDTH
    }
}

impl Viewable for ViewableCard {

    /// DON'T USE THIS!!!
    /// 
    /// Instead use view_and_move for a card with x & y at 0.0,
    /// because view alone does not calculate the correct offset.
    /// This calculation is moved to view_and_move, because otherwise
    /// some effects of the card won't render for an unknown reason.
    fn view<'a>(&self) -> Container<'a, AppMessage> {
        
        let img_opacity: f32 = f32_min_2(self.play_animation.get_opacity(),
                                         self.hide_animation.get_opacity());

        let hover_effect_opacity: f32 = f32_min_2(img_opacity,
                                                  self.focus_animation.get_opacity());

        let playable_opacity: f32 = f32_min_2(img_opacity,
                                              self.playable_animation.get_opacity());

        let false_played_opacity: f32 = f32_min_2(img_opacity,
                                               self.false_played_animation.get_opacity());

        let width: f32 = self.width() *
                         self.hover_animation.get_expansion() *
                         self.play_animation.get_contraction() *
                         self.hide_animation.get_contraction() *
                         self.draw_animation.get_contraction();

        let height: f32 = self.height() *
                          self.hover_animation.get_expansion();

        let rotation: f32 = self.rotation;

        // The factor 0.92 is chosen so the card will not get clipped when rotated.
        let scale: f32 = 0.92 * self.hide_animation.get_scale() * self.draw_animation.get_scale();


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
            let false_played_effect = image(FALSE_PLAYED_PATH)
                .content_fit(Fill)
                .width(width)
                .height(height)
                .rotation(rotation)
                .scale(scale)
                .opacity(false_played_opacity);
            card = card.push(false_played_effect)
        }

        let msg_hovered: AppMessage =
            ViewableCard::convert_to_app_message(CardMessage::Hovered(self.id));
        let msg_not_hoverd: AppMessage =
            ViewableCard::convert_to_app_message(CardMessage::NotHovered(self.id)
        );
        let msg_show_playable_status  =ViewableHand::convert_to_app_message(
            HandMessage::ShowPlayableStatus(self.show_playable_status.not())
        );
        let card_id: usize = self.id.clone();
        let msg_cursor_moved = move |position: Point|
            ViewableCard::convert_to_app_message(CardMessage::CursorMoved(card_id, position)
        );
        let msg_played =
            ViewableCard::convert_to_app_message(CardMessage::Played(self.id));
        let msg_false_played =
            ViewableCard::convert_to_app_message(CardMessage::FalsePlayed(self.id)
        );

        let mut mouse_area = 
            MouseArea::new(card)
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
        let x_offset: f32 = ((1.0 - self.play_animation.get_contraction()) / 2.0) *
                self.width() * self.hover_animation.get_expansion() +
                (self.width() - self.width() * self.hover_animation.get_expansion()) / 2.0;
        // y: realize the card moving up animation.
        let y_offset: f32 = - self.hover_animation.get_offset();
        let corrected_position: Point = Point::new(x + x_offset, y + y_offset);
        
        container(pin(self.view()).position(corrected_position))
    }
}