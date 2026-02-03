use iced::{
    Element, Font, widget::{
        Column, Row, button, column, container, image::Image, pick_list, row, scrollable, stack, text, text_input
    }
};


const TITLE_FONT: Font = Font::with_name("Magic School One");

use super::{App, AppMessage, MenuState, PlayerCount};
use crate::{api::{Card, Suit, Value}, ui_element_traits::Animated};
use crate::gameplay_ui::hand::{HandMessage, ViewableHand};
use crate::ui_element_traits::{Message, Viewable};
use derive_more::{Deref, DerefMut};

use crate::animation::{BasicAnimation, ReversableBasicAnimation, Easing, animation_end_sensor::AnimationEndSensor};
use iced::widget::MouseArea;

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
    pub fn new(id: usize, label: &'static str, img_path: &'static str, width: u16, height: u16) -> Self {
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
        let scale = self.hover_animation.get_expansion() * self.click_animation.get_contraction();
        let width_scaled: u16 = (self.width as f32 * scale).max(1.0).round() as u16;
        let height_scaled: u16 = (self.height as f32 * scale).max(1.0).round() as u16;

        let img = Image::new(self.img_path)
            .width(width_scaled as u32)
            .height(height_scaled as u32)
            .opacity(self.click_animation.get_opacity());

        let txt_size: u32 = ((height_scaled as f32) * 0.4) as u32;
        let mut s = stack!();
        s = s.push(img);
        s = s.push(
            container(text(self.label).size(txt_size))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
        );

        let base = container(s).width(self.width as u32).height(self.height as u32);
        let msg_hovered = AppMessage::ButtonMessage(ButtonMessage::Hovered(self.id));
        let msg_not_hovered = AppMessage::ButtonMessage(ButtonMessage::NotHovered(self.id));
        let msg_clicked = AppMessage::ButtonMessage(ButtonMessage::Clicked(self.id));

        let mouse_area = MouseArea::new(base)
            .on_enter(msg_hovered)
            .on_exit(msg_not_hovered)
            .on_press(msg_clicked)
            .interaction(iced::mouse::Interaction::Pointer);

        container(mouse_area)
    }

    pub fn view_with_label<'a>(&self, label: &'a str) -> container::Container<'a, AppMessage> {
        let scale = self.hover_animation.get_expansion() * self.click_animation.get_contraction();
        let width_scaled: u16 = (self.width as f32 * scale).max(1.0).round() as u16;
        let height_scaled: u16 = (self.height as f32 * scale).max(1.0).round() as u16;

        let img = Image::new(self.img_path)
            .width(width_scaled as u32)
            .height(height_scaled as u32)
            .opacity(self.click_animation.get_opacity());

        let txt_size: u32 = ((height_scaled as f32) * 0.4) as u32;
        let mut s = stack!();
        s = s.push(img);
        s = s.push(
            container(text(label).size(txt_size))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
        );

        let base = container(s).width(self.width as u32).height(self.height as u32);

        let msg_hovered = AppMessage::ButtonMessage(ButtonMessage::Hovered(self.id));
        let msg_not_hovered = AppMessage::ButtonMessage(ButtonMessage::NotHovered(self.id));
        let msg_clicked = AppMessage::ButtonMessage(ButtonMessage::Clicked(self.id));

        let mouse_area = MouseArea::new(base)
            .on_enter(msg_hovered)
            .on_exit(msg_not_hovered)
            .on_press(msg_clicked)
            .interaction(iced::mouse::Interaction::Pointer);

        container(mouse_area)
    }
}

impl Message for Button {
    type OwnMessage = ButtonMessage;

    fn convert_to_app_message(msg: ButtonMessage) -> AppMessage {
        AppMessage::ButtonMessage(msg)
    }

    fn update_with_msg(&mut self, msg: ButtonMessage) {
        match msg {
            ButtonMessage::Hovered(id) => {
                if id == self.id {
                    self.hover_animation.start();
                }
            }
            ButtonMessage::NotHovered(id) => {
                if id == self.id {
                    self.hover_animation.reverse();
                }
            }
            ButtonMessage::Clicked(id) => {
                if id == self.id {
                    self.click_animation.start();
                    self.click_end_sensor.start(Some(id));
                }
            }
        }
    }
}

impl Button {
    pub fn view_disabled(&self) -> container::Container<'_, AppMessage> {
        let width_scaled: u16 = self.width;
        let height_scaled: u16 = self.height;
        let img = Image::new(self.img_path)
            .width(width_scaled as u32)
            .height(height_scaled as u32)
            .opacity(0.6);
        let txt_size: u32 = ((height_scaled as f32) * 0.4) as u32;
        let mut s = stack!();
        s = s.push(img);
        s = s.push(
            container(text(self.label).size(txt_size).color(iced::Color::from_rgb(0.5,0.5,0.5)))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x(iced::Fill)
                .center_y(iced::Fill),
        );
        container(s).width(self.width as u32).height(self.height as u32)
    }
}

impl Animated for Button {
    fn update_animations(&mut self) {
        self.hover_animation.next_frame();
        self.click_animation.next_frame();
    }
}

/// Format a card for display (e.g., "5 Red", "Wizard", "Jester")
fn format_card(card: &Card) -> String {
    let value_str = match card.value {
        Value::Jester => "Jester".to_string(),
        Value::Wizard => "Wizard".to_string(),
        Value::Number(n) => n.to_string(),
    };

    match card.value {
        Value::Jester | Value::Wizard => value_str,
        Value::Number(_) => format!("{}\n{:?}", value_str, card.suit),
    }
}

/// Read PNG image dimensions from the file header
fn png_dimensions(path: &str) -> Option<(u32, u32)> {
    use std::fs::File;
    use std::io::Read;

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

fn menu_panel<'a>(
    state: &'a App,
    title: &'a str,
    body: Element<'a, AppMessage>,
    footer: Option<Element<'a, AppMessage>>) -> Element<'a, AppMessage> {
    let (intr_w, intr_h) = png_dimensions("assets/menu_container.png").unwrap_or((560, 440));
    let max_w = (state.window_size.width * 0.9) as u32;
    let max_h = (state.window_size.height * 0.9) as u32;
    let scale = ((max_w as f32) / (intr_w as f32)).min((max_h as f32) / (intr_h as f32)).min(1.0);
    let menu_w: u32 = (intr_w as f32 * scale).round() as u32;
    let menu_h: u32 = (intr_h as f32 * scale).round() as u32;

    let title_top_offset: u32 = std::cmp::max(((menu_h as f32) * 0.11).round() as u32, 12u32);

    let vertical_pad: u32 = 12;
    let side_padding: f32 = 36.0;

    let top_extra: u32 = ((menu_h as f32) * 0.22).round() as u32;

    let inner_w: u32 = (menu_w * 85 / 100).max(100);

    let inner = column![
        container(Column::new()).height(top_extra),
        body,
        footer.unwrap_or_else(|| container(Column::new()).into()),
    ]
    .spacing(10)
    .padding([vertical_pad as f32, side_padding])
    .width(inner_w)
    .height(menu_h);

    stack![
        container(Image::new("assets/menu_container.png").width(menu_w).height(menu_h))
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

pub fn view(state: &App) -> Element<'_, AppMessage> {
    match state.menu {
        MenuState::Main => view_main_menu(state),
        MenuState::Host => view_host_menu(state),
        MenuState::Join => view_join_menu(state),
        MenuState::Rules => view_rules_menu(state),
        MenuState::Lobby => view_lobby_menu(state),
        MenuState::Playing => view_gameplay(state),
        MenuState::PlayingTest => view_test_gameplay(state),
    }
}

fn view_main_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let title = 
    text("Wizard").
    size(130).
    font(TITLE_FONT).
    align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Top);

    let menu_left: Column<'a, AppMessage> = column![
        state.btn_host.view().padding(10),
        state.btn_join.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Left);

    let menu_right: Column<'a, AppMessage> = column![
        state.btn_rules.view().padding(10),
        state.btn_exit.view().padding(10),
    ]
    .spacing(100)
    .align_x(iced::alignment::Horizontal::Right);

    stack![
        Image::new("assets/wizard_main_menu.png").width(iced::Length::Fill).height(iced::Length::Fill),
        
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
            ].align_y(iced::alignment::Vertical::Center)
    ].into()
}

fn view_host_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
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

    let footer = Some(container(row![state.btn_back.view().padding(6)])
        .height(56u32)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .into());


    return stack![
        Image::new("assets/background_forall.png").width(iced::Length::Fill).height(iced::Length::Fill),

        menu_panel(state, "Spiel hosten:", content.into(), footer)
    ].into()
}

fn view_join_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let can_join = !state.ip.is_empty() && !state.join_name.is_empty();
    let content = column![
        text("Join").size(30),
        text("Name:"),
        text_input("Your Name", &state.join_name).on_input(AppMessage::JoinNameChanged),
        text_input("Server Address", &state.ip).on_input(AppMessage::ServerAddressChanged),
        if can_join { state.btn_connect.view().padding(0) } else { state.btn_connect.view_disabled().padding(0) },
        text("Progress:"),
        //state.btn_back.view().padding(0),
    ]
    .spacing(10)
    .padding(20)
    .width(400)
    .align_x(iced::alignment::Horizontal::Left);

    let footer = Some(container(row![state.btn_back.view().padding(6)])
        .height(56u32)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .into());

    return stack![
        Image::new("assets/background_forall.png").width(iced::Length::Fill).height(iced::Length::Fill),
        
        menu_panel(state, "Spiel beitreten:", content.into(), footer)
    ].into()
}

fn view_rules_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {

    let rules_body = column![
        text("grundlagen:").size(20),
        text("Wizard ist ein Stichspiel, bei dem das Ziel ist, möglichst genau vorherzusagen,"),
        text("wie viele Stiche man pro Runde macht."),
        
        text("die Anzahl der Spieler bestimmt die Anzahl der gespielten Runden:"),
        text(" - 3 Spieler: 20 Runden"),
        text(" - 4 Spieler: 16 Runden"),
        text(" - 5 Spieler: 13 Runden"),
        text(" - 6 Spieler: 11 Runden"),
        text(""),
        text("karten:").size(20),
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
        text(""),
        text("Trumpf:").size(20),
        text("Ein Trumpf ist eine bestimmte Farbe, die im Wert über allen nicht-trumpf Farben steht"),
        text("wird also eine nicht-Trumpf 12 gelegt, und darauf eine Trumpf 10, gewinnt die Trumpf 10 den Stich"),
        text("die Trumpf-Farbe wird am Anfang jeder Runde festgelegt"),
        text(""),
        text(""),
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
        text(""),
        text("Punkte:").size(20),
        text("stimmt die Ansage, kriegt man 20 Punkte Plus die Anzahl an gewonnenen Stichen mal 10 "),
        text("also bei 5 angesagten und 5 gewonnenen: 20 + 10*5 = 70 Punkte"),
        text("stimmt die Ansage nicht, wird das Zehnfache der Abweichung von den eigenen Punkten abgezogen. "),
        text("also bei 5 angesagten und 7 gewonnenen: 2 zu viel -> 2*10 = 20 Punkte Abzug"),
        text(""),
        text("Ende:").size(20),
        text("  in der gesamten letzten Runde wird ohne Trumpf gespielt."),
        text("  ist Runde 20 zuende gespielt, gewinnt der Spieler mit den meisten Punkten"),
    ];

    let max_h = (state.window_size.height * 0.9) as u32;
    let rules_body_scroll = scrollable(rules_body).height((max_h as f32 * 0.62) as u32).width(iced::Length::Fill);
    let footer = Some(container(row![state.btn_back.view().padding(6)])
        .height(56u32)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .into());

    return stack![
        Image::new("assets/background_forall.png").width(iced::Length::Fill).height(iced::Length::Fill),
        menu_panel(state, "SPIELREGELN:", rules_body_scroll.into(), footer)
        ].into();
}

fn view_lobby_menu<'a>(state: &'a App) -> Element<'a, AppMessage> {
    if !state.connected {
        return stack![
            Image::new("assets/wizard_lobby_menu.png").width(iced::Length::Fill).height(iced::Length::Fill),

            container(column![
            text("Nicht verbunden zum Server. / IP wurde falsch eingegeben"),
            state.btn_back.view().padding(0)
        ])
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        ].into();
    }
    if let Some(lobby) = &state.lobby {
        let mut player_rows = Column::new().spacing(10);
        for p in &lobby.players {
            let ready_text = if p.ready { "Bereit" } else { "Nicht bereit" };
            let toggle = if Some(p.id) == state.my_id {
                state.btn_ready_owned.view_with_label(ready_text)
            } else {
                container(text(ready_text)).width(80)
            };
            let row = row![
                text(format!(
                    "{}{}",
                    if p.is_host { "(Host) " } else { "" },
                    p.name
                )),
                toggle
            ];
            player_rows = player_rows.push(row);
        }

        let mut chat_block = Column::new().spacing(5);
        for (sender, msg) in &lobby.chat {
            chat_block = chat_block.push(text(format!("{}: {}", sender, msg)));
        }

        let can_start = lobby.players.len() == state.host_player_count.to_usize()
            && lobby.players.iter().all(|p| p.ready);
        let host_id = lobby.players.iter().find(|p| p.is_host).map(|p| p.id).unwrap_or_default();
        let i_am_host = state.my_id.is_some() && state.my_id.unwrap() == host_id;
        let start_button_view = if can_start && i_am_host {
            state.btn_start_game.view().padding(0)
        } else {
            state.btn_start_game.view_disabled().padding(0)
        };
        let start_button = row![
            start_button_view,
            text(if !can_start {
                " (Warten auf Spieler...)"
            } else if state.my_id.is_some() && !i_am_host {
                " (Nur der Host kann starten)"
            } else {
                ""
            })
        ]
        .spacing(5);

        let content = column![
            text("Lobby").size(30),
            row![
                text("Host Address:"),
                text_input("Address to share", &state.ip)
            ]
            .spacing(10),
            text(format!(
                "Spieler: {}/{}",
                lobby.players.len(),
                state.host_player_count.to_usize()
            )),
            player_rows,
            scrollable(chat_block).height(150).width(400),
            row![
                text_input("Nachricht", &state.chat_input).on_input(AppMessage::ChatInputChanged),
                state.btn_send_chat.view().padding(0),
            ],
            start_button,
            state.btn_back_to_menu.view().padding(0)
        ]
        .spacing(10)
        .padding(20);
        
        stack![
            Image::new("assets/wizard_lobby_menu.png").width(iced::Length::Fill).height(iced::Length::Fill),

            container(content)
            .center_x(iced::Fill)
            .center_y(iced::Fill)
        ].into()
    } else {
        stack![
            Image::new("assets/wizard_lobby_menu.png").width(iced::Length::Fill).height(iced::Length::Fill),

            container(column![
                text("Keine Lobby vorhanden"),
                state.btn_back.view().padding(0)
            ])
            .center_x(iced::Fill)
            .center_y(iced::Fill)
        ].into()
    }
}

fn get_player_name(state: &App, player_id: u64) -> String {
    if state.my_id == Some(player_id) {
        return "You".to_string();
    }
    if let Some(ref lobby) = state.lobby {
        if let Some(player) = lobby.players.iter().find(|p| p.id == player_id) {
            return player.name.clone();
        }
    }
    format!("Player {}", player_id)
}

fn view_test_gameplay<'a>(state: &'a App) -> Element<'a, AppMessage> {
    let my_name = state
        .my_id
        .map(|id| get_player_name(state, id))
        .unwrap_or("?".to_string());
    let trump_str = state
        .trump
        .map(|s| format!("{:?}", s))
        .unwrap_or("None".to_string());
    let current_player_name = state
        .current_player
        .map(|id| get_player_name(state, id))
        .unwrap_or("?".to_string());

    let header = column![
        text(format!(
            "Round {} | Trump: {} | You: {}",
            state.round_number, trump_str, my_name
        ))
        .size(18),
        text(format!(
            "Current Player: {} | Phase: {}",
            current_player_name,
            if state.must_set_trump {
                "Set Trump"
            } else if state.is_bidding_phase {
                "Bidding"
            } else {
                "Playing"
            }
        ))
        .size(14),
        text(format!("Status: {}", state.last_msg)).size(12),
    ]
    .spacing(5);

    // Trump selection (if dealer needs to set)
    let trump_section: Element<'a, AppMessage> = if state.must_set_trump {
        column![
            text("SELECT TRUMP SUIT:").size(16),
            row![
                button("Red")
                    .on_press(AppMessage::SetTrump(Suit::Red))
                    .padding(8),
                button("Blue")
                    .on_press(AppMessage::SetTrump(Suit::Blue))
                    .padding(8),
                button("Green")
                    .on_press(AppMessage::SetTrump(Suit::Green))
                    .padding(8),
                button("Yellow")
                    .on_press(AppMessage::SetTrump(Suit::Yellow))
                    .padding(8),
            ]
            .spacing(5),
        ]
        .spacing(5)
        .into()
    } else {
        text("").into()
    };

    // Bidding section
    let bidding_section: Element<'a, AppMessage> =
        if state.is_bidding_phase && state.is_my_turn && !state.must_set_trump {
            column![
                text("YOUR BID:").size(16),
                row![
                    text_input("Enter bid", &state.bid_input)
                        .on_input(AppMessage::BidInputChanged)
                        .width(80),
                    button("Submit Bid")
                        .on_press(AppMessage::SubmitBid)
                        .padding(8),
                ]
                .spacing(5),
                text(format!("(0 to {})", state.round_number + 1)).size(12),
            ]
            .spacing(5)
            .into()
        } else {
            text("").into()
        };

    let bids_display: Element<'a, AppMessage> = if !state.bids.is_empty() {
        let mut bids_col = Column::new().spacing(2);
        bids_col = bids_col.push(text("Tricks / Bids:").size(14));
        for (player_id, bid) in &state.bids {
            let player_name = get_player_name(state, *player_id);
            let tricks = state.tricks_won.get(player_id).unwrap_or(&0);
            bids_col =
                bids_col.push(text(format!("  {}: {} / {}", player_name, tricks, bid)).size(12));
        }
        bids_col.into()
    } else {
        text("").into()
    };

    let trick_display: Element<'a, AppMessage> = if !state.current_trick.is_empty() {
        let mut trick_col = Column::new().spacing(2);
        trick_col = trick_col.push(text("Current Trick:").size(14));
        for (player_id, card) in &state.current_trick {
            let player_name = get_player_name(state, *player_id);
            let card_str = format_card(card);
            trick_col = trick_col.push(
                text(format!(
                    "  {} played {}",
                    player_name,
                    card_str.replace('\n', " ")
                ))
                .size(12),
            );
        }
        trick_col.into()
    } else {
        text("Trick: (empty)").size(12).into()
    };

    let hand_section: Element<'a, AppMessage> = {
        let mut hand_col = Column::new().spacing(5);
        hand_col = hand_col.push(text(format!("Your Hand ({} cards):", state.hand.len())).size(16));

        let mut card_row = Row::new().spacing(5);
        for card in &state.hand {
            let card_text = format_card(card);
            let is_valid = state.valid_cards.is_empty() || state.valid_cards.contains(card);
            let can_play =
                state.is_my_turn && !state.is_bidding_phase && !state.must_set_trump && is_valid;

            let card_btn = if can_play {
                button(text(card_text).size(11))
                    .on_press(AppMessage::PlayCard(*card))
                    .padding(8)
            } else {
                button(text(card_text).size(11)).padding(8)
            };
            card_row = card_row.push(card_btn);
        }
        hand_col = hand_col.push(scrollable(card_row).direction(
            scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
        ));
        hand_col.into()
    };

    let scores_section: Element<'a, AppMessage> = {
        let mut scores_col = Column::new().spacing(2);
        scores_col = scores_col.push(text("Scores:").size(14));
        for player_id in &state.player_order {
            let player_name = get_player_name(state, *player_id);
            let score = state.scores.get(player_id).unwrap_or(&0);
            scores_col =
                scores_col.push(text(format!("  {}: {} pts", player_name, score)).size(12));
        }
        scores_col.into()
    };

    let game_over_section: Element<'a, AppMessage> = if state.game_over {
        let winner_name = state
            .winner
            .map(|id| get_player_name(state, id))
            .unwrap_or("Unknown".to_string());
        let is_me = state.my_id == state.winner;
        column![
            text(if is_me {
                "🎉 YOU WON! 🎉"
            } else {
                "GAME OVER"
            })
            .size(24),
            text(format!("Winner: {}", winner_name)).size(18),
            state.btn_back_to_menu.view().padding(8),
        ]
        .spacing(10)
        .into()
    } else {
        text("").into()
    };

    let log_section: Element<'a, AppMessage> = {
        let mut log_col = Column::new().spacing(2);
        log_col = log_col.push(text("Game Log:").size(14));
        let start = if state.game_log.len() > 15 {
            state.game_log.len() - 15
        } else {
            0
        };
        for entry in state.game_log.iter().skip(start) {
            log_col = log_col.push(text(entry).size(10));
        }
        scrollable(log_col).height(150).into()
    };

    let players_section: Element<'a, AppMessage> = if !state.player_order.is_empty() {
        let mut players_str = String::from("Players: ");
        for (i, pid) in state.player_order.iter().enumerate() {
            let is_current = state.current_player == Some(*pid);
            let player_name = get_player_name(state, *pid);
            players_str.push_str(&format!(
                "{}{}{}",
                if is_current { "[" } else { "" },
                player_name,
                if is_current { "]" } else { "" }
            ));
            if i < state.player_order.len() - 1 {
                players_str.push_str(" → ");
            }
        }
        text(players_str).size(12).into()
    } else {
        text("").into()
    };

    let content = column![
        text("WIZARD").size(24),
        game_over_section,
        header,
        players_section,
        trump_section,
        bidding_section,
        bids_display,
        trick_display,
        hand_section,
        scores_section,
        log_section,
        button("Back to Menu")
            .on_press(AppMessage::BackToMenu)
            .padding(8),
    ]
    .spacing(10)
    .padding(20);

    container(scrollable(content))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

fn view_gameplay<'a>(state: &'a App) -> Element<'a, AppMessage> {
    column![
        state.viewable_hand.view(),
        button("Draw Cards").on_press(ViewableHand::convert_to_app_message(
            HandMessage::DrawCards(ViewableHand::build_test_cards(state.window_size))
        ))
    ]
    .into()
}
