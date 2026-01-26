//#![allow(unused_variables)]
//#![allow(dead_code)]

use futures::{SinkExt, StreamExt};
use iced::{
    Element, Size, Subscription, Task, time,
    widget::{button, column, row, text, text_input},
    window
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

use crate::{api::{C, ServerMessage}, game_elements::GameElement};
use crate::game_elements::hand::{Hand, HandMessage};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;

#[derive(Debug)]
struct App {
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    msg: String,
    ip: String,

    window_size: Size,
    hand: Hand
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
            hand: Hand::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Host,
    Join,
    Ip(String),
    Tick,

    WindowResized(Size),
    AnimationTick,
    HandMessage(HandMessage)
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
        AppMessage::HandMessage(hand_msg) => {
            state.hand.update_with_msg(hand_msg);
            Task::none()
        }
        AppMessage::AnimationTick => {
            state.hand.update_animations();
            Task::none()
        }
        AppMessage::WindowResized(size) => {
            state.window_size = size;
            state.hand.update_size(size);
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
        column![state.hand.view(),
                button("Draw Cards")
                .on_press(
                    Hand::convert_to_app_message(HandMessage::DrawCards(Hand::build_test_cards()))
                )
               ].into()
    }
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    let mut subscriptions: Vec<Subscription<AppMessage>> = vec!();
    subscriptions.push(window::resize_events()
                       .map(|(_, size)| AppMessage::WindowResized(size)));
    subscriptions.push(time::every(Duration::from_millis(16))
                       .map(|_| AppMessage::AnimationTick));
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
