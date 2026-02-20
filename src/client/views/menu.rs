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
        text("Host").size(30),
        text("Name:"),
        text_input("Your Name", &state.host_name).on_input(AppMessage::HostNameChanged),
        text("Player Count:"),
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
        text("Join").size(30),
        text("Name:"),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        if can_join {
            state.btn_connect.view().padding(0)
        } else {
            state.btn_connect.view_disabled().padding(0)
        },
        text("Progress:"),
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
        text("Grundlagen:").size(20),
        text("Wizard ist ein Stichspiel, bei dem das Ziel ist, möglichst genau vorherzusagen,"),
        text("wie viele Stiche man pro Runde macht."),
        text("Die Anzahl der Spieler bestimmt die Anzahl der gespielten Runden:"),
        text(" - 3 Spieler: 20 Runden"),
        text(" - 4 Spieler: 16 Runden"),
        text(" - 5 Spieler: 13 Runden"),
        text(" - 6 Spieler: 11 Runden"),
        text(""),
        text("Karten:").size(20),
        text("Das Wizard Deck besteht aus 60 Karten:"),
        text(" - Zahlen 1-13 - Kreuz"),
        text(" - Zahlen 1-13 - Pik"),
        text(" - Zahlen 1-13 - Herz"),
        text(" - Zahlen 1-13 - Karo"),
        text(" - 4 Wizards"),
        text(" - 4 Narren"),
        text(""),
        text("Stiche:").size(20),
        text("Ein Stich wird von der höchsten Karte, oder dem ersten gelegten Wizard gewonnen"),
        text("\n"),
        text("Trumpf:").size(20),
        text("Ein Trumpf ist eine bestimmte Farbe, die im Wert über allen nicht-trumpf Farben steht"),
        text("wird also eine nicht-Trumpf 12 gelegt, und darauf eine Trumpf 10, gewinnt die Trumpf 10 den Stich"),
        text("die Trumpf-Farbe wird am Anfang jeder Runde festgelegt"),
        text("\n"),
        text("\n"),
        text("Spielablauf:").size(24),
        text("Anfang:").size(20),
        text("In der ersten Runde bekommt jeder Spieler genau eine Karte, blabla Placeholder..."),
        text("Jede Runde in Wizard hat denselben Ablauf, der Trumpf wird aufgedeckt und jeder Spieler bekommt,"),
        text("der Rundenzahl entsprechend viele Karten (also in Runde 5 -> 5 Karten, in Runde 12 -> 12 Karten...)"),
        text("als nächstes gibt jeder Spieler an, wie viele Stiche er diese Runde machen wird"),
        text("ACHTUNG! - Die Gesamtzahl aller angesagten Stiche kann nie gleich mit den möglichen Stichen sein."),
        text("Anschließend spielt jeder der Reihe nach genau eine Karte."),
        text("Hat jeder Spieler genau eine Karte gelegt, beginnt der Gewinner dieses Stichs den nächsten Stich."),
        text("Sind alle Karten ausgespielt, werden Stiche mit den Ansagen abgeglichen und entsprechend Punkte verteilt."),
        text("\n"),
        text("Punkte:").size(20),
        text("stimmt die Ansage, kriegt man 20 Punkte Plus die Anzahl an gewonnenen Stichen mal 10 "),
        text("also bei 5 angesagten und 5 gewonnenen: 20 + 10*5 = 70 Punkte"),
        text("stimmt die Ansage nicht, wird das Zehnfache der Abweichung von den eigenen Punkten abgezogen. "),
        text("also bei 5 angesagten und 7 gewonnenen: 2 zu viel -> 2*10 = 20 Punkte Abzug"),
        text("\n"),
        text("Ende:").size(20),
        text("  in der gesamten letzten Runde wird ohne Trumpf gespielt."),
        text("  ist Runde 20 zuende gespielt, gewinnt der Spieler mit den meisten Punkten"),
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
