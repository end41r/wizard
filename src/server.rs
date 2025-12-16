use axum::{
    routing::get,
    response::IntoResponse,
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    Router,
};
use std::net::SocketAddr;
use uuid::Uuid;
use crate::api::{C, S, B};

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

// Helper to broadcast to all clients (TODO: implement actual broadcast)
async fn broadcast_to_all(_broadcast: B) {
    // TODO: Store all client sockets and send to each
    println!("Broadcasting: {:?}", _broadcast);
}

async fn handle_socket(mut socket: WebSocket) {
    let id = Uuid::new_v4().as_u128() as u64;
    println!("New connection: player {}", id);

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<C>(&text) {
                Ok(C::JoinLobby { name }) => {
                    println!("Player {} joining lobby as {}", id, name);
                    let response = S::JoinConfirmation { ok: true };
                    let _ = socket
                        .send(Message::Text(serde_json::to_string(&response).unwrap()))
                        .await;
                    
                    // Broadcast updated lobby state
                    broadcast_to_all(B::LobbyState { 
                        players: vec![] // TODO: actual player list
                    }).await;
                }
                Ok(C::LeaveLobby) => {
                    println!("Player {} leaving lobby", id);
                    
                    // Broadcast updated lobby state
                    broadcast_to_all(B::LobbyState { 
                        players: vec![] // TODO: actual player list
                    }).await;
                    break;
                }
                Ok(C::SetReady { ready }) => {
                    println!("Player {} set ready: {}", id, ready);
                    
                    // Broadcast updated lobby state
                    broadcast_to_all(B::LobbyState { 
                        players: vec![] // TODO: actual player list
                    }).await;
                }
                Ok(C::Bid { amount }) => {
                    println!("Player {} bid: {}", id, amount);
                    
                    // Broadcast the bid
                    broadcast_to_all(B::BidMade { 
                        player: id, 
                        amount 
                    }).await;
                    // TODO: Validate bid, check if bidding is complete
                }
                Ok(C::PlayCard { card }) => {
                    println!("Player {} played card: {:?}", id, card);
                    
                    // Broadcast the card played
                    broadcast_to_all(B::CardPlayed { 
                        player: id, 
                        card: card.clone() 
                    }).await;
                    // TODO: Validate play, check if pool is complete
                }
                Err(err) => {
                    println!("Parse error from player {}: {}", id, err);
                    let error = S::Error { 
                        reason: format!("Invalid message: {}", err) 
                    };
                    let _ = socket
                        .send(Message::Text(serde_json::to_string(&error).unwrap()))
                        .await;
                }
            }
        }
    }
    
    println!("Player {} disconnected", id);
}
