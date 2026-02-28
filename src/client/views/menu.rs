use iced::{
    widget::{
        column, container, pick_list, row, scrollable, slider, stack, text, text_input, Column,
        Image, Row,
    },
    ContentFit, Element,
};

use super::utils::{back_button_footer, background_image, menu_panel, TITLE_FONT};
use crate::api::TextColor;
use crate::client::{App, AppMessage, PlayerCount};

pub fn view_main_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let title = text("Wizard")
        .size(130)
        .font(TITLE_FONT)
        .white()
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Top);

    let menu_left: Column<'a, AppMessage> = column![
        state.btn_host.view().padding(10),
        state.btn_join.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Left);

    let menu_right: Column<'a, AppMessage> = column![
        state.btn_options.view().padding(10),
        state.btn_close.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Right);
    let bottom: Row<'a, AppMessage> = row![if state.disconnected {
        text("You have been disconnected from the server. Please check your connection and try again.")
                .size(20)
                .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                .white()
    } else {
        text("")
    }];
    stack![
        Image::new(state.img_main_menu.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(ContentFit::Cover),
        container(title)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Fill),
        row![
            container(menu_left)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
            container(menu_right)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
        ]
        .align_y(iced::alignment::Vertical::Center),
        container(bottom)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Fill)
            .align_y(iced::alignment::Vertical::Bottom),
    ]
    .into()
}

pub fn view_host_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let count_options = vec![
        PlayerCount::P3,
        PlayerCount::P4,
        PlayerCount::P5,
        PlayerCount::P6,
    ];

    let can_join = !state.host_name.is_empty();

    let content = column![
        text("Host").size(30).white(),
        text("Name:").white(),
        text_input("Your Name", &state.host_name).on_input(AppMessage::HostNameChanged),
        text("Player Count:").white(),
        pick_list(
            count_options.clone(),
            Some(state.host_player_count),
            AppMessage::HostPlayerCountChanged
        ),
        if can_join {
            state.btn_create_lobby.view().padding(0)
        } else {
            state.btn_create_lobby.view_disabled().padding(0)
        },
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    stack![
        background_image(&state.img_background),
        menu_panel(
            state,
            "Spiel hosten:",
            content.into(),
            back_button_footer(state)
        )
    ]
    .into()
}

pub fn view_join_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let can_join = !state.ip.is_empty() && !state.join_name.is_empty();

    let content = column![
        text("Join").size(30).white(),
        text("Name:").white(),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        if can_join {
            state.btn_connect.view().padding(0)
        } else {
            state.btn_connect.view_disabled().padding(0)
        },
        text("Progress:").white(),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    stack![
        background_image(&state.img_background),
        menu_panel(
            state,
            "Spiel beitreten:",
            content.into(),
            back_button_footer(state)
        )
    ]
    .into()
}

pub fn view_options_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let content = column![
        text("Musiklautstärke:").white(),
        slider(
            0.0..=100.0,
            state.music_volume as f32,
            AppMessage::MusicVolumeChanged
        )
        .step(1.0)
        .width(iced::Length::Fill),
        text(format!("{}%", state.music_volume)).white(),
        text("SFX Lautstärke:").white(),
        slider(
            0.0..=100.0,
            state.sfx_volume as f32,
            AppMessage::SfxVolumeChanged
        )
        .step(1.0)
        .width(iced::Length::Fill),
        text(format!("{}%", state.sfx_volume)).white(),
        state.btn_rules.view().padding(0),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    stack![
        background_image(&state.img_background),
        menu_panel(
            state,
            "Optionen:",
            content.into(),
            back_button_footer(state)
        )
    ]
    .into()
}

pub fn view_rules_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let rules_body = column![
        text("Grundlagen:").size(20).white(),
        text("Wizard ist ein Stichspiel, bei dem das Ziel ist, möglichst genau vorherzusagen,").white(),
        text("wie viele Stiche man pro Runde macht.").white(),
        text("Die Anzahl der Spieler bestimmt die Anzahl der gespielten Runden:").white(),
        text(" - 3 Spieler: 20 Runden").white(),
        text(" - 4 Spieler: 16 Runden").white(),
        text(" - 5 Spieler: 13 Runden").white(),
        text(" - 6 Spieler: 11 Runden").white(),
        text("").white(),
        text("Karten:").size(20).white(),
        text("Das Wizard Deck besteht aus 60 Karten:").white(),
        text(" - Zahlen 1-13 - Kreuz").white(),
        text(" - Zahlen 1-13 - Pik").white(),
        text(" - Zahlen 1-13 - Herz").white(),
        text(" - Zahlen 1-13 - Karo").white(),
        text(" - 4 Wizards").white(),
        text(" - 4 Jester").white(),
        text("").white(),
        text("Stiche:").size(20).white(),
        text("Ein Stich wird von der höchsten Karte, oder dem ersten gelegten Wizard gewonnen").white(),
        text("\n").white(),
        text("Trumpf:").size(20).white(),
        text("Ein Trumpf ist eine bestimmte Farbe, die im Wert über allen nicht-trumpf Farben steht").white(),
        text("wird also eine nicht-Trumpf 12 gelegt, und darauf eine Trumpf 10, gewinnt die Trumpf 10 den Stich").white(),
        text("die Trumpf-Farbe wird am Anfang jeder Runde festgelegt").white(),
        text("\n").white(),
        text("\n").white(),
        text("Spielablauf:").size(24).white(),
        text("Jede Runde in Wizard hat denselben Ablauf:").size(20).white(),
        text("Am Anfang jeder Runde wird ein Dealer und der Trumpf bestimmt.").white(),
        text("Wenn der Trumpf ein Wizard ist, wird die Trumpf-Farbe vom Dealer festgelegt.").white(),
        text("Ist der Trumpf ein Narr, gibt es in dieser Runde keinen Trumpf").white(),
        text("Danach setzt jeder Spieler seinen Bid, also die Anzahl der Tricks, die er in dieser Runde gewinnen muss.").white(),
        text("Es werden so viele Karten ausgeteilt, wie die aktuelle Rundennummer angibt, also in Runde 1 eine Karte, in Runde 2 zwei Karten, etc.").white(),
        text("ACHTUNG! - Die Gesamtzahl aller angesagten Stiche darf nicht gleich der Anzahl der möglichen Stiche sein.").white(),
        text("Anschließend spielt jeder der Reihe nach genau eine Karte.").white(),
        text("Hat jeder Spieler genau eine Karte gelegt, beginnt der Gewinner dieses Stichs den nächsten Stich.").white(),
        text("Sind alle Karten ausgespielt, werden Stiche mit den Ansagen abgeglichen und die entsprechenden Punkte verteilt.").white(),
        text("\n").white(),
        text("Punkte:").size(20).white(),
        text("Hat ein Spieler genau die Anzahl an Stichen gewonnen, die er angesagt hat, bekommt er 20 Punkte plus 10 Punkte pro gewonnenem Stich.").white(),
        text("Beispiel: 3 Stiche angesagt, 3 Stiche gewonnen -> 20 + 10*3 = 50 Punkte").white(),
        text("Hat ein Spieler nicht die Anzahl an Stichen gewonnen, die er angesagt hat, bekommt er -10 Punkte pro zu viel oder zu wenig gewonnenem Stich.").white(),
        text("Beispiel: 3 Stiche angesagt, 5 Stiche gewonnen -> -10 * (5-3) = -20 Punkte").white(),
        text("\n").white(),
        text("Ende:").size(20).white(),
        text("In der letzten Runde gibt es keinen Trumpf").white(),
        text("Es werden so viele Runden gespielt bis alle Karten ausgeteilt wurden, also bei 3 Spielern 20 Runden, bei 4 Spielern 15 Runden, etc.").white(),
        text("Der Spieler mit den meisten Punkten am Ende des Spiels gewinnt!").white()
    ];

    let max_h = (state.window_size.height * 0.9) as u32;
    let rules_body_scroll = scrollable(rules_body)
        .height((max_h as f32 * 0.62) as u32)
        .width(iced::Length::Fill);

    stack![
        background_image(&state.img_background),
        menu_panel(
            state,
            "SPIELREGELN:",
            rules_body_scroll.into(),
            back_button_footer(state)
        )
    ]
    .into()
}
