use iced::{
    widget::{button, column, text},
    Element, Subscription, Task,
};
use serde::{Serialize, Deserialize};
use futures::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ClientMsg {
    Join { name: String },
    SendHello { message: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ServerMsg {
    Welcome { id: u64 },
    GameUpdate { state: String },
    JoinConfirmation { ok: bool },
    Error { message: String },
}

type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<ClientMsg>>>>;
type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMsg>>>>;

#[derive(Debug)]
struct App {
    started: bool,
    connected: bool,
    ws_tx: WsConnection,
    server_rx: ServerMsgReceiver,
    last_msg: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            started: false,
            connected: false,
            ws_tx: Arc::new(Mutex::new(None)),
            server_rx: Arc::new(Mutex::new(None)),
            last_msg: "Not connected".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    Connect,
    SendHello,
    CheckMessages,
}

fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    if !state.started {
        let _ = crate::server::start_server();
        state.started = true;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    match msg {
        AppMessage::Connect => {
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        connect_ws(ws_tx, server_rx).await;
                    });
                });
                state.connected = true;
                state.last_msg = "Connecting...".to_string();
            }
            // Start checking messages
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::SendHello => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(ClientMsg::SendHello { message : "Ping".to_string() });
                    state.last_msg = "Ping".to_string();
                }
            }
            Task::done(AppMessage::CheckMessages)
        }
        AppMessage::CheckMessages => {
            println!("CheckMessages called!");
            if let Ok(guard) = state.server_rx.lock() {
                if let Some(ref rx) = *guard {
                    while let Ok(msg) = rx.try_recv() {
                        println!("Received: {:?}", msg);
                        match msg {
                            ServerMsg::Welcome { id } => {
                                state.last_msg = format!("Welcome! Your ID: {}", id);
                            }
                            ServerMsg::GameUpdate { state: game_state } => {
                                state.last_msg = format!("From server: {}", game_state);
                            }
                            ServerMsg::Error { message } => {
                                state.last_msg = format!("Error: {}", message);
                            }
                            ServerMsg::JoinConfirmation { ok } => {
                                state.last_msg = format!("Join Confirmation: {}", ok);
                            }
                       }
                    }
                } else {
                    println!("No receiver yet");
                }
            }

            Task::future(async {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                AppMessage::CheckMessages
            })
        }
    }
}

async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver) {
    let (ws_stream, _) = connect_async("ws://127.0.0.1:3000/ws").await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (srv_tx, srv_rx) = std::sync::mpsc::channel();

    *ws_tx.lock().unwrap() = Some(tx);
    *server_rx.lock().unwrap() = Some(srv_rx);

    // Send join message
    let join_msg = ClientMsg::Join { name: "Player".into() };
    let _ = write.send(WsMessage::Text(serde_json::to_string(&join_msg).unwrap())).await;

    // Send task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if write.send(WsMessage::Text(text)).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(WsMessage::Text(txt))) = read.next().await {
        if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&txt) {
            let _ = srv_tx.send(server_msg);
        }
    }
}

fn view(state: &'_ App) -> Element<'_, AppMessage> {
    column![
        button("Connect").on_press(AppMessage::Connect),
        button("Send Ping").on_press(AppMessage::SendHello),
        text(format!("Status: {}", state.last_msg)),

    ]
    .padding(20)
    .into()
}

fn subscription(_state: &App) -> Subscription<AppMessage> {
    Subscription::none()
}pub fn main() -> iced::Result {
    iced::application("Wizard", update, view)
        .subscription(subscription)
        .run()
}
