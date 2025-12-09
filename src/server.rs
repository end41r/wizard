use axum::{
    routing::get,
    response::IntoResponse,
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    Router,
};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use uuid::Uuid;

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

pub fn start_server() -> Result<(), String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_server().await;
        });
    });
    Ok(())
}

async fn run_server() {
    let app = Router::new().route("/ws", get(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let id = Uuid::new_v4().as_u128() as u64;

    let welcome = ServerMsg::JoinConfirmation { ok:true };
    let _ = socket
        .send(Message::Text(serde_json::to_string(&welcome).unwrap()))
        .await;

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMsg>(&text) {
                Ok(ClientMsg::Join { name }) => {
                    println!("player {id} joined as {name}");

                    let confirmation = ServerMsg::JoinConfirmation { ok: true };
                    let _ = socket
                        .send(Message::Text(serde_json::to_string(&confirmation).unwrap()))
                        .await;
                }
                Ok(ClientMsg::SendHello { message }) => {
                    println!("player {id} sent hello: {}", message);

                    // broadcast back only to this client
                    let update = ServerMsg::GameUpdate {
                        state: format!("Pong"),
                    };
                    let _ = socket
                        .send(Message::Text(serde_json::to_string(&update).unwrap()))
                        .await;
                }
                Err(err) => {
                    let err_msg = ServerMsg::Error {
                        message: format!("Parse error: {err}"),
                    };
                    let _ = socket
                        .send(Message::Text(serde_json::to_string(&err_msg).unwrap()))
                        .await;
                }
            }
        }
    }
}
