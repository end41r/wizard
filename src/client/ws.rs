use crate::api::ServerMessage;
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

use crate::api::C;

pub type WsConnection = Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<C>>>>;
pub type ServerMsgReceiver = Arc<Mutex<Option<std::sync::mpsc::Receiver<ServerMessage>>>>;

pub async fn connect_ws(ws_tx: WsConnection, server_rx: ServerMsgReceiver, ip: String) {
    let url = format!("ws://{ip}:3000/ws");
    println!("Attempting to connect to {url}...");
    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            println!("WebSocket started!");
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (srv_tx, srv_rx) = std::sync::mpsc::channel();

            *ws_tx.lock().unwrap() = Some(tx);
            *server_rx.lock().unwrap() = Some(srv_rx);
            println!("Receiver set successfully!");

            // Send task.
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if write.send(WsMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
            });

            // Receive task and directly parse it as a ServerMessage.
            let recv_task = tokio::spawn(async move {
                while let Some(Ok(WsMessage::Text(txt))) = read.next().await {
                    println!("Raw message received: {txt}");
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        println!("Parsed successfully: {server_msg:?}");
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

            if let Ok(mut w) = ws_tx.lock() {
                *w = None;
            }
            if let Ok(mut s) = server_rx.lock() {
                *s = None;
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {e}");
        }
    }
}
