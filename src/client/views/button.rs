use derive_more::{Deref, DerefMut};
use iced::widget::{container, stack, text, Image, MouseArea};

use crate::animation::{
    animation_end_sensor::AnimationEndSensor, BasicAnimation, Easing, ReversableBasicAnimation,
};
use crate::client::AppMessage;
use crate::ui_element_traits::{Animated, Message};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct HoverAnim(ReversableBasicAnimation);

impl HoverAnim {
    pub fn new(duration: usize) -> Self {
        Self(ReversableBasicAnimation::new(duration))
    }

    pub fn get_expansion(&self) -> f32 {
        let progress = self.progress(Easing::InOutCubic);
        progress * 0.05 + 1.0
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct ClickAnim(BasicAnimation);

impl ClickAnim {
    pub fn new(duration: usize) -> Self {
        Self(BasicAnimation::new(duration))
    }

    pub fn get_contraction(&self) -> f32 {
        1.0 - self.progress(Easing::InCubic) * 0.1
    }

    pub fn get_opacity(&self) -> f32 {
        1.0 - self.progress(Easing::Linear) * 0.4
    }
}

#[derive(Debug, Clone)]
pub enum ButtonMessage {
    Hovered(usize),
    NotHovered(usize),
    Clicked(usize),
}

#[derive(Debug)]
pub struct Button {
    pub id: usize,
    pub label: &'static str,
    img_path: &'static str,
    width: u16,
    height: u16,
    hover_animation: HoverAnim,
    click_animation: ClickAnim,
    click_end_sensor: AnimationEndSensor<usize>,
}

impl Button {
    pub fn new(
        id: usize,
        label: &'static str,
        img_path: &'static str,
        width: u16,
        height: u16,
    ) -> Self {
        let click_duration: usize = 15;
        Self {
            id,
            label,
            img_path,
            width,
            height,
            hover_animation: HoverAnim::new(12),
            click_animation: ClickAnim::new(click_duration),
            click_end_sensor: AnimationEndSensor::new(click_duration),
        }
    }

    pub fn check_click_end<F>(&mut self, action: F) -> bool
    where
        F: FnOnce(&usize),
    {
        let finished = self.click_end_sensor.check(|h| {
            if let Some(k) = h.content() {
                action(k);
            }
        });
        if finished {
            self.click_animation.reset();
        }
        finished
    }

    pub fn view(&self) -> container::Container<'_, AppMessage> {
        self.view_internal(self.label)
    }

    pub fn view_with_label<'a>(&self, label: &'a str) -> container::Container<'a, AppMessage> {
        self.view_internal(label)
    }

    fn view_internal<'a>(&self, label: &'a str) -> container::Container<'a, AppMessage> {
        let scale = self.hover_animation.get_expansion() * self.click_animation.get_contraction();
        let width_scaled = (self.width as f32 * scale).max(1.0).round() as u16;
        let height_scaled = (self.height as f32 * scale).max(1.0).round() as u16;
        let txt_size = ((height_scaled as f32) * 0.4) as u32;

        let img = Image::new(self.img_path)
            .width(width_scaled as u32)
            .height(height_scaled as u32)
            .opacity(self.click_animation.get_opacity());

        let content = stack![
            img,
            container(text(label).size(txt_size))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
        ];

        let base = container(content)
            .width(self.width as u32)
            .height(self.height as u32);

        let mouse_area = MouseArea::new(base)
            .on_enter(AppMessage::ButtonMessage(ButtonMessage::Hovered(self.id)))
            .on_exit(AppMessage::ButtonMessage(ButtonMessage::NotHovered(
                self.id,
            )))
            .on_press(AppMessage::ButtonMessage(ButtonMessage::Clicked(self.id)))
            .interaction(iced::mouse::Interaction::Pointer);

        container(mouse_area)
    }

    pub fn view_disabled(&self) -> container::Container<'_, AppMessage> {
        self.view_disabled_internal(self.label)
    }

    pub fn view_disabled_with_label<'a>(
        &self,
        label: &'a str,
    ) -> container::Container<'a, AppMessage> {
        self.view_disabled_internal(label)
    }

    fn view_disabled_internal<'a>(&self, label: &'a str) -> container::Container<'a, AppMessage> {
        let txt_size = ((self.height as f32) * 0.4) as u32;

        let img = Image::new(self.img_path)
            .width(self.width as u32)
            .height(self.height as u32)
            .opacity(0.6);

        let content = stack![
            img,
            container(
                text(label)
                    .size(txt_size)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Fill)
            .center_y(iced::Fill),
        ];

        container(content)
            .width(self.width as u32)
            .height(self.height as u32)
    }
}

/// Implements traits
impl Message for Button {
    type OwnMessage = ButtonMessage;

    fn convert_to_app_message(msg: ButtonMessage) -> AppMessage {
        AppMessage::ButtonMessage(msg)
    }

    fn update_with_msg(&mut self, msg: ButtonMessage) {
        match msg {
            ButtonMessage::Hovered(id) if id == self.id => {
                self.hover_animation.start();
            }
            ButtonMessage::NotHovered(id) if id == self.id => {
                self.hover_animation.reverse();
            }
            ButtonMessage::Clicked(id) if id == self.id => {
                self.click_animation.start();
                self.click_end_sensor.start(Some(id));
            }
            _ => {}
        }
    }
}

impl Animated for Button {
    fn update_animations(&mut self) {
        self.hover_animation.next_frame();
        self.click_animation.next_frame();
    }
}
