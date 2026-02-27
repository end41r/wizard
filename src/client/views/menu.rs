use iced::{
    widget::{
        column, container, pick_list, row, scrollable, stack, text, text_input, Column, Image,
    },
    ContentFit, Element,
};

use crate::client::{App, AppMessage, PlayerCount};

use super::utils::{back_button_footer, background_image, menu_panel, TITLE_FONT};

pub fn view_main_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let title = text("Wizard")
        .size(130)
        .font(TITLE_FONT)
        .color(iced::Color::WHITE)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Top);

    let menu_left: Column<'a, AppMessage> = column![
        state.btn_host.view().padding(10),
        state.btn_join.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Left);

    let menu_right: Column<'a, AppMessage> = column![
        state.btn_rules.view().padding(10),
        state.btn_close.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Right);

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
        .align_y(iced::alignment::Vertical::Center)
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
        text("Host").size(30).color(iced::Color::WHITE),
        text("Name:").color(iced::Color::WHITE),
        text_input("Your Name", &state.host_name).on_input(AppMessage::HostNameChanged),
        text("Player Count:").color(iced::Color::WHITE),
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
        text("Join").size(30).color(iced::Color::WHITE),
        text("Name:").color(iced::Color::WHITE),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        if can_join {
            state.btn_connect.view().padding(0)
        } else {
            state.btn_connect.view_disabled().padding(0)
        },
        text("Progress:").color(iced::Color::WHITE),
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

pub fn view_rules_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let rules_body = column![
        text("Grundlagen:").size(20).color(iced::Color::WHITE),
        text("Wizard ist ein Stichspiel, bei dem das Ziel ist, möglichst genau vorherzusagen,").color(iced::Color::WHITE),
        text("wie viele Stiche man pro Runde macht.").color(iced::Color::WHITE),
        text("Die Anzahl der Spieler bestimmt die Anzahl der gespielten Runden:").color(iced::Color::WHITE),
        text(" - 3 Spieler: 20 Runden").color(iced::Color::WHITE),
        text(" - 4 Spieler: 16 Runden").color(iced::Color::WHITE),
        text(" - 5 Spieler: 13 Runden").color(iced::Color::WHITE),
        text(" - 6 Spieler: 11 Runden").color(iced::Color::WHITE),
        text("").color(iced::Color::WHITE),
        text("Karten:").size(20).color(iced::Color::WHITE),
        text("Das Wizard Deck besteht aus 60 Karten:").color(iced::Color::WHITE),
        text(" - Zahlen 1-13 - Kreuz").color(iced::Color::WHITE),
        text(" - Zahlen 1-13 - Pik").color(iced::Color::WHITE),
        text(" - Zahlen 1-13 - Herz").color(iced::Color::WHITE),
        text(" - Zahlen 1-13 - Karo").color(iced::Color::WHITE),
        text(" - 4 Wizards").color(iced::Color::WHITE),
        text(" - 4 Narren").color(iced::Color::WHITE),
        text("").color(iced::Color::WHITE),
        text("Stiche:").size(20).color(iced::Color::WHITE),
        text("Ein Stich wird von der höchsten Karte, oder dem ersten gelegten Wizard gewonnen").color(iced::Color::WHITE),
        text("\n"),
        text("Trumpf:").size(20).color(iced::Color::WHITE),
        text("Ein Trumpf ist eine bestimmte Farbe, die im Wert über allen nicht-trumpf Farben steht").color(iced::Color::WHITE),
        text("wird also eine nicht-Trumpf 12 gelegt, und darauf eine Trumpf 10, gewinnt die Trumpf 10 den Stich").color(iced::Color::WHITE),
        text("die Trumpf-Farbe wird am Anfang jeder Runde festgelegt").color(iced::Color::WHITE),
        text("\n").color(iced::Color::WHITE),
        text("\n").color(iced::Color::WHITE),
        text("Spielablauf:").size(24).color(iced::Color::WHITE),
        text("Anfang:").size(20).color(iced::Color::WHITE),
        text("In der ersten Runde bekommt jeder Spieler genau eine Karte, blabla Placeholder...").color(iced::Color::WHITE),
        text("Jede Runde in Wizard hat denselben Ablauf, der Trumpf wird aufgedeckt und jeder Spieler bekommt,"),
        text("der Rundenzahl entsprechend viele Karten (also in Runde 5 -> 5 Karten, in Runde 12 -> 12 Karten...)").color(iced::Color::WHITE),
        text("als nächstes gibt jeder Spieler an, wie viele Stiche er diese Runde machen wird").color(iced::Color::WHITE),
        text("ACHTUNG! - Die Gesamtzahl aller angesagten Stiche kann nie gleich mit den möglichen Stichen sein.").color(iced::Color::WHITE),
        text("Anschließend spielt jeder der Reihe nach genau eine Karte.").color(iced::Color::WHITE),
        text("Hat jeder Spieler genau eine Karte gelegt, beginnt der Gewinner dieses Stichs den nächsten Stich.").color(iced::Color::WHITE),
        text("Sind alle Karten ausgespielt, werden Stiche mit den Ansagen abgeglichen und entsprechend Punkte verteilt.").color(iced::Color::WHITE),
        text("\n"),
        text("Punkte:").size(20).color(iced::Color::WHITE),
        text("stimmt die Ansage, kriegt man 20 Punkte Plus die Anzahl an gewonnenen Stichen mal 10 ").color(iced::Color::WHITE),
        text("also bei 5 angesagten und 5 gewonnenen: 20 + 10*5 = 70 Punkte").color(iced::Color::WHITE),
        text("stimmt die Ansage nicht, wird das Zehnfache der Abweichung von den eigenen Punkten abgezogen. ").color(iced::Color::WHITE),
        text("also bei 5 angesagten und 7 gewonnenen: 2 zu viel -> 2*10 = 20 Punkte Abzug").color(iced::Color::WHITE),
        text("\n"),
        text("Ende:").size(20).color(iced::Color::WHITE),
        text("  in der gesamten letzten Runde wird ohne Trumpf gespielt.").color(iced::Color::WHITE),
        text("  ist Runde 20 zuende gespielt, gewinnt der Spieler mit den meisten Punkten").color(iced::Color::WHITE),
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
