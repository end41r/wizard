use derive_more::{Deref, DerefMut};
use iced::widget::{container, stack, text, Image, MouseArea};
use iced::Task;

use crate::animation::{BasicAnimation, Easing, ReversableBasicAnimation};
use crate::api::BUTTON1_PATH;
use crate::client::{AppMessage, MenuState, TaskBatcher};
use crate::ui_element_traits::{Animated, Message, Notifiable};

#[derive(Debug, Clone)]
pub enum ButtonMessage {
    Hovered(usize),
    NotHovered(usize),
    Clicked(usize),
    ClickEnded(usize),
}

impl Message for ButtonMessage {
    fn convert_msg_from(msg: Self) -> AppMessage {
        AppMessage::ButtonMessage(msg)
    }
}

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

#[derive(Debug)]
pub struct Button {
    pub id: usize,
    pub text: &'static str,
    img_path: &'static str,
    width: u16,
    height: u16,
    hover_animation: HoverAnim,
    click_animation: ClickAnim,
    on_click: AppMessage,
}

impl Button {
    fn new(
        id: usize,
        text: &'static str,
        img_path: &'static str,
        width: u16,
        height: u16,
        on_click: AppMessage,
    ) -> Self {
        let mut button = Self {
            id,
            text,
            img_path,
            width,
            height,
            hover_animation: HoverAnim::new(12),
            click_animation: ClickAnim::new(15),
            on_click,
        };
        button
            .click_animation
            .on_end(ButtonMessage::ClickEnded(id).convert_msg());
        button
    }

    pub fn new_host_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(id,"Host", BUTTON1_PATH, width, heigth, AppMessage::Host)
    }
    pub fn new_join_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Beitreten",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::Navigate(MenuState::Join),
        )
    }
    pub fn new_rules_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Spielregeln",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::GameRules,
        )
    }
    pub fn new_close_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Spiel verlassen",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::CloseGame,
        )
    }
    pub fn new_create_lobby_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Lobby erstellen",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::CreateLobby,
        )
    }
    pub fn new_ready_owned_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Bereit",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::ToggleReady(id),
        )
    }
    pub fn new_back_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Zurück",
            BUTTON1_PATH,
            width,
            heigth,
            MenuState::Main.convert_msg(),
        )
    }
    pub fn new_connect_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Verbinden",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::Connect,
        )
    }
    pub fn new_send_chat_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Senden",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::SendChat,
        )
    }
    pub fn new_start_game_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Starten",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::StartGame,
        )
    }
    pub fn new_back_to_menu_button(id: usize, width: u16, heigth: u16) -> Self {
        Self::new(
            id,
            "Zurück zum Menü",
            BUTTON1_PATH,
            width,
            heigth,
            AppMessage::BackToMenu,
        )
    }

    pub fn view(&self) -> container::Container<'_, AppMessage> {
        self.view_internal(self.text)
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
        self.view_disabled_internal(self.text)
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

impl Notifiable for Button {
    type OwnMessage = ButtonMessage;

    fn update_with_msg(&mut self, msg: ButtonMessage) -> Task<AppMessage> {
        match msg {
            ButtonMessage::Hovered(id) => {
                if id == self.id {
                    self.hover_animation.start()
                }
            }
            ButtonMessage::NotHovered(id) => {
                if id == self.id {
                    self.hover_animation.reverse()
                }
            }
            ButtonMessage::Clicked(id) => {
                if id == self.id {
                    self.click_animation.start();
                }
            }
            ButtonMessage::ClickEnded(id) => {
                if id == self.id {
                    self.click_animation.reset();
                    return self.on_click.convert_msg_to_task();
                }
            }
        }
        Task::none()
    }
}

impl Animated for Button {
    fn update_animations(&mut self) -> Task<AppMessage> {
        TaskBatcher::instant_batch([
            self.hover_animation.next_frame(),
            self.click_animation.next_frame(),
        ])
    }
}
