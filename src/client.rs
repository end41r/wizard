//#![allow(unused_variables)]
//#![allow(dead_code)]

use iced::{
    widget::{button, column, text, text_input, row},
    time, Element, Subscription, Task,
};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};

use crate::api::{C, ServerMessage};

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;

#[derive(Debug)]
struct App {
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    msg: String,
    ip: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            msg: String::new(),
            ip: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Host,
    Join,
    Ip(String),
    Tick,
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
        AppMessage::Ip(v) => { state.ip = v; Task::none() }
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
        // send C messages if needed
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

            // Send task
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if write.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
            });

            // Receive task - parse as ServerMessage directly
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

            // Wait for either task to complete
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
    column![
        button("Host").on_press(AppMessage::Host),
        row![text_input("IP", &state.ip).on_input(AppMessage::Ip), button("Join").on_press(AppMessage::Join)].spacing(5),
        text(&state.msg),
    ].spacing(10).padding(20).into()
}

fn subscription(state: &App) -> Subscription<AppMessage> {
    // Use iced's time::every for polling - this is the correct way
    if state.connected {
        time::every(Duration::from_millis(100)).map(|_| AppMessage::Tick)
    } else {
        Subscription::none()
    }
}

pub fn main() -> iced::Result {
    iced::application("Wizard", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(300.0, 300.0),
            resizable: true,
            ..Default::default()
        })
        .run()
}
