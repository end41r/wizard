use std::fs::File;
use std::io::Read;

use iced::{
    widget::{column, container, row, stack, text, Column, Image},
    Element, Font,
};

use crate::client::{App, AppMessage};

pub const TITLE_FONT: Font = Font::with_name("Magic School One");

use crate::api::{Card, Value};

/// Format a card for display (e.g., "5 Red", "Wizard", "Jester").
pub fn format_card(card: &Card) -> String {
    match card.value {
        Value::Jester => "Jester".to_string(),
        Value::Wizard => "Wizard".to_string(),
        Value::Number(n) => format!("{}\n{:?}", n, card.suit),
    }
}

pub fn get_player_name(state: &App, player_id: u64) -> String {
    if state.my_id == Some(player_id) {
        return "You".to_string();
    }

    state
        .lobby
        .as_ref()
        .and_then(|lobby| lobby.players.iter().find(|p| p.id == player_id))
        .map(|player| player.name.clone())
        .unwrap_or_else(|| format!("Player {}", player_id))
}

/// Read PNG image dimensions from the file header.
pub fn png_dimensions(path: &str) -> Option<(u32, u32)> {
    let mut f = File::open(path).ok()?;
    let mut buf = [0u8; 24];
    f.read_exact(&mut buf).ok()?;

    if &buf[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }

    let width = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let height = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
    Some((width, height))
}

/// Creates a full-screen background with the specified image.
pub fn background_image(path: &'static str) -> Image<iced::widget::image::Handle> {
    Image::new(path)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
}

pub fn menu_panel<'a>(
    state: &'a App,
    title: &'a str,
    body: Element<'a, AppMessage>,
    footer: Option<Element<'a, AppMessage>>,
) -> Element<'a, AppMessage> {
    let (intr_w, intr_h) = png_dimensions("assets/menu_container.png").unwrap_or((560, 440));
    let max_w = (state.window_size.width * 0.9) as u32;
    let max_h = (state.window_size.height * 0.9) as u32;

    let scale = ((max_w as f32) / (intr_w as f32))
        .min((max_h as f32) / (intr_h as f32))
        .min(1.0);

    let menu_w = (intr_w as f32 * scale).round() as u32;
    let menu_h = (intr_h as f32 * scale).round() as u32;
    let title_top_offset = ((menu_h as f32) * 0.11).round().max(12.0) as u32;
    let top_extra = ((menu_h as f32) * 0.22).round() as u32;
    let inner_w = (menu_w * 85 / 100).max(100);

    let inner = column![
        container(Column::new()).height(top_extra),
        body,
        footer.unwrap_or_else(|| container(Column::new()).into()),
    ]
    .spacing(10)
    .padding([12.0, 36.0])
    .width(inner_w)
    .height(menu_h);

    stack![
        container(
            Image::new("assets/menu_container.png")
                .width(menu_w)
                .height(menu_h)
        )
        .center_x(iced::Fill)
        .center_y(iced::Fill),
        container(
            Column::new()
                .width(menu_w)
                .height(menu_h)
                .push(container(Column::new()).height(title_top_offset))
                .push(
                    container(
                        text(title)
                            .size(38)
                            .font(TITLE_FONT)
                            .color(iced::Color::from_rgb(0.0, 0.0, 0.0)),
                    )
                    .height(48u32)
                    .center_x(iced::Fill),
                ),
        )
        .center_x(iced::Fill),
        container(inner).center_x(iced::Fill).center_y(iced::Fill)
    ]
    .into()
}

/// Creates a back button footer for menu panels.
pub fn back_button_footer<'a>(state: &'a App) -> Option<Element<'a, AppMessage>> {
    Some(
        container(row![state.btn_back.view().padding(6)])
            .height(56u32)
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .into(),
    )
}
