use axum::{
    routing::get,
    response::IntoResponse,
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    Router,
};
use futures::{StreamExt, SinkExt};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use crate::api::{C, S, B, ServerMessage};

type Clients = Arc<RwLock<HashMap<u64, mpsc::UnboundedSender<ServerMessage>>>>;

pub fn local_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0").ok()
        .and_then(|s| { s.connect("8.8.8.8:80").ok()?; s.local_addr().ok() })
        .map(|a| a.ip().to_string())
        .unwrap_or("?".into())
}

pub fn start_server() -> Result<(), String> {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
            run_server(clients).await;
        });
    });
    Ok(())
}

async fn run_server(clients: Clients) {
    let app = Router::new()
        .route("/ws", get({
            let clients = clients.clone();
            move |ws| ws_handler(ws, clients.clone())
        }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, clients: Clients) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

// Broadcast to all connected clients
async fn broadcast_to_all(clients: &Clients, broadcast: B) {
    let wrapped = ServerMessage::Broadcast(broadcast);
    let message_text = serde_json::to_string(&wrapped).unwrap();
    println!("Broadcasting: {}", message_text);
    let clients_map = clients.read().await;
    for (client_id, tx) in clients_map.iter() {
        if tx.send(wrapped.clone()).is_err() {
            println!("Failed to send to client {}", client_id);
        }
    }
}

async fn handle_socket(socket: WebSocket, clients: Clients) {
    let id = Uuid::new_v4().as_u128() as u64;
    println!("New connection: player {}", id);
    
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    
    // Register client
    clients.write().await.insert(id, tx.clone());
    
    // Spawn task to forward messages from channel to websocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if sender.send(Message::Text(text)).await.is_err() {
                        println!("Failed to send to player {}", id);
                        break;
                    }
                }
                Err(e) => {
                    println!("Serialization error for player {}: {}", id, e);
                    break;
                }
            }
        }
    });
    
    // Main receive loop
    let clients_clone = clients.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<C>(&text) {
                    Ok(C::JoinLobby { name }) => {
                        println!("Player {} joining lobby as {}", id, name);
                        let response = ServerMessage::Server(S::JoinConfirmation { ok: true });
                        if let Err(e) = tx.send(response) {
                            println!("Failed to send join confirmation: {:?}", e);
                        }
                        
                        // Broadcast updated lobby state
                        broadcast_to_all(&clients_clone, B::LobbyState { 
                            players: vec![] // TODO: actual player list
                        }).await;
                    }
                    Ok(C::LeaveLobby) => {
                        println!("Player {} leaving lobby", id);
                        
                        // Broadcast updated lobby state
                        broadcast_to_all(&clients_clone, B::LobbyState { 
                            players: vec![] // TODO: actual player list
                        }).await;
                    }
                    Ok(C::SetReady { ready }) => {
                        println!("Player {} set ready: {}", id, ready);
                        
                        // Broadcast updated lobby state
                        broadcast_to_all(&clients_clone, B::LobbyState { 
                            players: vec![] // TODO: actual player list
                        }).await;
                    }
                    Ok(C::Bid { amount }) => {
                        println!("Player {} bid: {}", id, amount);
                        
                        // Broadcast the bid
                        broadcast_to_all(&clients_clone, B::BidMade { 
                            player: id, 
                            amount 
                        }).await;
                        // TODO: Validate bid, check if bidding is complete
                    }
                    Ok(C::PlayCard { card }) => {
                        println!("Player {} played card: {:?}", id, card);
                        
                        // Broadcast the card played
                        broadcast_to_all(&clients_clone, B::CardPlayed { 
                            player: id, 
                            card: card.clone() 
                        }).await;
                        // TODO: Validate play, check if pool is complete
                    }
                    Err(err) => {
                        println!("Parse error from player {}: {}", id, err);
                        let error = ServerMessage::Server(S::Error { 
                            reason: format!("Invalid message: {}", err) 
                        });
                        let _ = tx.send(error);
                    }
                }
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
    
    // Cleanup: remove client
    clients.write().await.remove(&id);
    println!("Player {} disconnected", id);
}
