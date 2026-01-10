//#![allow(unused_variables)]
//#![allow(dead_code)]

use futures::{SinkExt, StreamExt};
use iced::{
    Element, mouse::Interaction, Point, Size, Subscription, Task, time,
    widget::{MouseArea, Pin, Stack, button, column,
             container, image, pin, row, stack, text, text_input},
    window
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use indexmap::{IndexMap, map::MutableKeys};

use crate::api::{ServerMessage, C};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;

static CARD1_PATH:&'static str = "assets/cards/1.png";
static CARD2_PATH:&'static str = "assets/cards/2.png";
static CARD3_PATH:&'static str = "assets/cards/3.png";
static CARD4_PATH:&'static str = "assets/cards/4.png";
static MULT_BASE_WIDTH_CARD_WIDTH: f32 = 0.12;
static MULT_BASE_WIDTH_CARD_HEIGHT: f32 = MULT_BASE_WIDTH_CARD_WIDTH * 1.46;
static MULT_BASE_WIDTH_CARD_STACK_OFFSET: f32 = MULT_BASE_WIDTH_CARD_HEIGHT * 0.53;

#[derive(Debug)]
struct App {
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    msg: String,
    ip: String,

    window_size: Size,
    cards: IndexMap<usize, Card>,
    card_base_size: Size,
    focus_card_row_low: bool,
    top_card_id_upper: usize,
    top_card_id_lower: usize 
}

impl Default for App {

    fn default() -> Self {
        Self {
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            msg: String::new(),
            ip: String::new(),

            window_size: Size::new(300.0, 300.0),
            cards: IndexMap::from([
                (0, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (1, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
                (2, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (3, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (4, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (5, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (6, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (7, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
                (8, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (9, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (10, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (11, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (12, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (13, Card::new(CARD4_PATH, Size::new(154.0, 225.0))),
                (14, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
                (15, Card::new(CARD1_PATH, Size::new(154.0, 225.0))),
                (16, Card::new(CARD3_PATH, Size::new(154.0, 225.0))),
                (17, Card::new(CARD2_PATH, Size::new(154.0, 225.0))),
            ]),
            card_base_size: Size::new(154.0, 225.0),
            focus_card_row_low: true,
            top_card_id_upper: 100, // Impossible to reach
            top_card_id_lower: 100  // Impossible to reach
        }
    }
}

impl App {

    fn get_card(&self, id: usize) -> &Card {
        self.cards.get(&id).unwrap()
    }

    fn get_card_mut(&mut self, id: usize) -> &mut Card {
        self.cards.get_mut(&id).unwrap()
    }

    fn get_card_ids(&self) -> Vec<usize> {
        let mut card_ids: Vec<usize> = vec!();
        for card in self.cards.iter() {
            card_ids.push(*card.0);
        }
        card_ids
    }

    fn get_hand_width(&self) -> f32 {
        // 10 cards layered size with max width reached while left and right card at max size
        self.card_base_size.width * 3.0 +
        self.card_base_size.width * 1.1
    }

    fn get_hand_height(&self) -> f32 {
        // This does ignore size multiplication of cards sinze their dimensions move down
        // while the card is moving up so the total length of the card hand is not altered by that.
        self.card_base_size.height -  // card bottom
        self.get_hand_row_distance() +  // card top
        self.card_base_size.height * 0.15  // card offset
    }

    fn get_hand_width_offset(&self) -> f32 {
        (self.card_base_size.width * 1.1 - self.card_base_size.width) / 2.0
    }

    fn get_hand_height_offset(&self) -> f32 {
        self.get_hand_height() - self.card_base_size.height
    }

    fn get_hand_row_distance(&self) -> f32 {
        -self.window_size.width * MULT_BASE_WIDTH_CARD_STACK_OFFSET
    }

    fn get_card_width(&self) -> f32 {
        self.window_size.width * MULT_BASE_WIDTH_CARD_WIDTH
    }

    fn get_card_height(&self) -> f32 {
        self.window_size.width * MULT_BASE_WIDTH_CARD_HEIGHT
    }

    fn get_upper_row_card_spawn_point(&self) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let row_y_offset: f32 = self.get_hand_row_distance();

        let mut row_x_offset: f32 = 0.0;
        if self.cards.len() > 10 {
            let cards_in_row: usize = self.cards.len() - 10;
            let row_len: f32 = (cards_in_row as f32) * self.card_base_size.width / 3.0 +
                                     self.card_base_size.width * (2.0/3.0);
            row_x_offset = (max_row_len - row_len) / 2.0;
        }

        Point::new(row_x_offset, row_y_offset)
    }

    fn get_lower_row_card_spawn_point(&self) -> Point {

        let max_row_len: f32 = 4.0 * self.card_base_size.width;

        let cards_in_row: usize = std::cmp::min(self.cards.len(), 10);
        let row_len: f32 = (cards_in_row as f32) * self.card_base_size.width / 3.0 +
                           self.card_base_size.width * (2.0/3.0);
        let row_x_offset: f32 = (max_row_len - row_len) / 2.0;
        let row_y_offset: f32 = 0.0;

        Point::new(row_x_offset, row_y_offset)

    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Host,
    Join,
    Ip(String),
    Tick,

    WindowResized(Size),
    CardPlayed(usize),
    CardHovered(usize),
    CardNotHovered(usize),
    FrameTick,
}

#[derive(Debug)]
struct Card {
    img_path: &'static str,
    offset: f32,
    size: Size,
    size_mult: f32,
    moving_up: CardMoveState,
}

impl Card {

    fn new(img_path: &'static str, size: Size) -> Self{
        Card {
            img_path: img_path,
            size: size,
            size_mult: 1.0,
            offset: 0.0,
            moving_up: CardMoveState::NotMoving
        }
    }

    fn move_card_up(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.moving_up == CardMoveState::MovingUp && self.offset <= max_card_offset {
            self.size_mult += 0.02;
            self.offset += max_card_offset * 0.2;
        }
        else if self.moving_up != CardMoveState::MovingDown {
            self.moving_up = CardMoveState::NotMoving;
        }
    }

    fn move_card_down(&mut self) {
        let max_card_offset: f32 = self.size.height * 0.15;
        if self.moving_up == CardMoveState::MovingDown && self.offset > 0.0 {
            self.size_mult -= 0.02;
            self.offset -= max_card_offset * 0.20;
        }
        else if self.moving_up != CardMoveState::MovingUp {
            self.moving_up = CardMoveState::NotMoving;
            // Correcting floating point error
            self.size_mult = 1.0;
            self.offset = 0.0;
        }
    }
}

#[derive(PartialEq, Debug)]
enum CardMoveState {
    MovingUp,
    MovingDown,
    NotMoving
}

fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    match msg {
        AppMessage::Host => {
            if !state.connected {
                let _ = crate::server::start_server();
                std::thread::sleep(std::time::Duration::from_millis(300));
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, "127.0.0.1".into()));
                });
                state.connected = true;
                state.msg = format!("Hosting on {}", crate::server::local_ip());
            }
            Task::none()
        }
        AppMessage::Join => {
            if !state.connected && !state.ip.is_empty() {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                let ip = state.ip.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, ip));
                });
                state.connected = true;
                state.msg = "Connecting...".into();
            }
            Task::none()
        }
        AppMessage::Ip(v) => {
            state.ip = v;
            Task::none()
        }
        AppMessage::Tick => {
            if let Ok(g) = state.server_rx.lock() {
                if let Some(ref rx) = *g {
                    while let Ok(m) = rx.try_recv() {
                        // handle S, B messages
                        state.msg = format!("{:?}", m);
                    }
                }
            }
            Task::none()
        }
        AppMessage::CardHovered(card_id) => {
            state.get_card_mut(card_id).moving_up = CardMoveState::MovingUp;
            if state.cards.len() > 10 && state.get_card_ids()[..state.cards.len() - 10]
                                                             .contains(&card_id) {
                    state.focus_card_row_low = false;
                    state.top_card_id_upper = card_id;
                } else {
                    state.focus_card_row_low = true;
                    state.top_card_id_lower = card_id;
                }
            Task::none()
        }
        AppMessage::CardPlayed(card_id) => {
            println!("Card with id {} played!", card_id);
            Task::none()
        }
        AppMessage::CardNotHovered(card_id) => {
            state.get_card_mut(card_id).moving_up = CardMoveState::MovingDown;
            Task::none()
        }
        AppMessage::FrameTick => {
            for card_id in state.get_card_ids().iter() {
                if state.get_card(*card_id).moving_up == CardMoveState::MovingUp {
                    state.get_card_mut(*card_id).move_card_up();
                }
                if state.get_card(*card_id).moving_up == CardMoveState::MovingDown {
                    state.get_card_mut(*card_id).move_card_down();
                }
                if *card_id != state.top_card_id_lower &&
                   *card_id != state.top_card_id_upper &&
                   state.get_card_mut(*card_id).offset != 0.0 {
                        state.get_card_mut(*card_id).move_card_down();
                }
            };
            Task::none()
        }
        AppMessage::WindowResized(size) => {
            state.window_size = size;
            state.card_base_size = Size::new(state.get_card_width(),
                                             state.get_card_height());
            for (_, card) in state.cards.iter_mut2() {
                card.size = state.card_base_size;
            };
            Task::none()
        } // Send C messages if needed
    }
}

async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver, ip: String) {
    let url = format!("ws://{}:3000/ws", ip);
    println!("Attempting to connect to {}...", url);
    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            println!("WebSocket connected!");
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (srv_tx, srv_rx) = std::sync::mpsc::channel();

            *ws_tx.lock().unwrap() = Some(tx);
            *server_rx.lock().unwrap() = Some(srv_rx);
            println!("Receiver set successfully!");

            // Send a task.
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if write.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
            });

            // Recieve a task and directly parse it as a ServerMessage.
            let recv_task = tokio::spawn(async move {
                while let Some(Ok(WsMessage::Text(txt))) = read.next().await {
                    println!("Raw message received: {}", txt);
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        println!("Parsed successfully: {:?}", server_msg);
                        let _ = srv_tx.send(server_msg);
                    } else {
                        println!("Failed to parse message");
                    }
                }
                println!("Receive loop ended");
            });

            // Wait for either task to complete.
            tokio::select! {
                _ = send_task => println!("Send task ended"),
                _ = recv_task => println!("Receive task ended"),
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
        }
    }
}

fn view_card<'a>(card_id: &usize, card: &Card, x_pos: f32, y_pos: f32) -> Pin<'a, AppMessage> {
        pin(
            (MouseArea::new(image(card.img_path)
                     .width(card.size.width * card.size_mult)
                     .height(card.size.height * card.size_mult)))
                     .on_double_click(AppMessage::CardPlayed(*card_id))
                     .on_enter(AppMessage::CardHovered(*card_id))
                     .on_exit(AppMessage::CardNotHovered(*card_id))
                     .interaction(Interaction::Pointer)
        )
        .position(Point::new(x_pos + (card.size.width - card.size.width * card.size_mult) / 2.0,
                             y_pos-(card.offset as f32)))
    }

fn view_hand<'a>(state: &App) -> Pin<'a, AppMessage> {

    // Create a stack for the whole hand and another two for the upper/lower row.
    let mut card_stack: Stack<'_, AppMessage> = stack!();
    let mut card_stack_upper: Stack<'_, AppMessage> = stack!();
    let mut card_stack_lower: Stack<'_, AppMessage> = stack!();

    // Push all cards in state.cards to their row.
    let mut x_pos: f32 = 0.0;
    let y_pos: f32 = 0.0;
    let x_pos_offset: f32 = state.get_hand_width_offset();
    let y_pos_offset: f32 = state.get_hand_height_offset();
    let mut move_card_stack_lower = true;
    if state.cards.len() > 10 {
        move_card_stack_lower = false;
    }
    let mut push_lower = false;

    for (i, (card_id, card)) in state.cards.iter().enumerate() {

        let viewable_card: Pin<'_, AppMessage> = view_card(card_id, card,
            x_pos + x_pos_offset, y_pos + y_pos_offset);

        if move_card_stack_lower {
            if push_lower {
                card_stack_lower = card_stack_lower.push_under(viewable_card)
            } else {
                card_stack_lower = card_stack_lower.push(viewable_card)
            }
        } else {
            if push_lower {
                card_stack_upper = card_stack_upper.push_under(viewable_card)
            } else {
                card_stack_upper = card_stack_upper.push(viewable_card)
            }
        }

        if (!move_card_stack_lower && *card_id == state.top_card_id_upper) ||  // Top card reached
           (move_card_stack_lower && *card_id == state.top_card_id_lower) {
                push_lower = true;
        }

        x_pos = x_pos + card.size.width / 3.0;

        if state.cards.len() > 10 && i + 1 == state.cards.len() - 10 {  // Row switch
            x_pos = 0.0;
            push_lower = false;
            move_card_stack_lower = true;
        }
    }

    // Add the upper/lower row to the whole hand.
    if state.focus_card_row_low {
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(state.get_upper_row_card_spawn_point()));
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(state.get_lower_row_card_spawn_point()));
        } else {
            card_stack = card_stack.push(pin(card_stack_lower)
                                            .position(state.get_lower_row_card_spawn_point()));
            card_stack = card_stack.push(pin(card_stack_upper)
                                            .position(state.get_upper_row_card_spawn_point()));
        }

    pin(card_stack).width(state.get_hand_width()).height(state.get_hand_height())
}

fn view(state: &'_ App) -> Element<'_, AppMessage> {
    let gui_switch: usize = 0;
    if gui_switch == 1 {
        column![
            button("Host").on_press(AppMessage::Host),
            row![
                text_input("IP", &state.ip).on_input(AppMessage::Ip),
                button("Join").on_press(AppMessage::Join)
            ]
            .spacing(5),
            text(&state.msg),
        ]
        .spacing(10)
        .padding(20)
        .into()
    } else {
        container(view_hand(state)).into()
    }
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    let mut subscriptions: Vec<Subscription<AppMessage>> = vec!();
    subscriptions.push(window::resize_events()
                       .map(|(_, size)| AppMessage::WindowResized(size)));
    subscriptions.push(time::every(Duration::from_millis(16))
                       .map(|_| AppMessage::FrameTick));
    if state.connected {
        subscriptions.push(time::every(Duration::from_millis(100)).map(|_| AppMessage::Tick));
    };
    Subscription::batch(subscriptions)
}

pub fn main() -> iced::Result {
    iced::application(App::default, update, view)
    .title("Wizard")
    .subscription(subscription)
    .window_size(Size::new(300.0, 300.0))
    .run()
}
