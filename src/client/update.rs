use iced::{clipboard, Task};
use std::sync::Arc;

use super::{connect_ws, App, AppMessage, MenuState, PlayerCount};
use crate::api::{Lobby, ServerMessage, B, C, S};

pub fn update(state: &mut App, msg: AppMessage) -> Task<AppMessage> {
    match msg {
        AppMessage::Navigate(menu) => {
            state.menu = menu;
            Task::none()
        }
        AppMessage::Host => {
            let local_ip = crate::server::local_ip();
            state.msg = format!("Hosting on {local_ip}");
            state.ip = local_ip;

            state.menu = MenuState::Host;
            Task::none()
        }
        AppMessage::HostNameChanged(name) => {
            state.host_name = name;
            Task::none()
        }
        AppMessage::HostPlayerCountChanged(count) => {
            state.host_player_count = count;
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetPlayerCount {
                        count: count.to_usize(),
                    });
                    state.last_msg = format!("Player count set to {count}");
                }
            }
            Task::none()
        }
        AppMessage::JoinNameChanged(name) => {
            state.join_name = name;
            Task::none()
        }
        AppMessage::ServerAddressChanged(addr) => {
            state.ip = addr;
            Task::none()
        }
        AppMessage::CopyToClipboard(addr) => {
            state.last_msg = "Server address copied to clipboard.".to_string();
            clipboard::write(addr)
        }

        AppMessage::SendChat => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::ChatMessage {
                        sender: state.join_name.clone(),
                        message: state.chat_input.clone(),
                    });
                    state.last_msg = "Sending chat message...".to_string();
                    state.chat_input.clear();
                }
            }
            Task::none()
        }
        AppMessage::ChatInputChanged(input) => {
            state.chat_input = input;
            Task::none()
        }
        AppMessage::CreateLobby => {
            if !state.connected {
                crate::server::start_server();
                std::thread::sleep(std::time::Duration::from_millis(300));
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, "127.0.0.1".into()));
                });
                state.connected = true;
            }

            println!("Creating lobby...");
            std::thread::sleep(std::time::Duration::from_millis(2000));

            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby {
                        name: state.host_name.clone(),
                    });
                    state.last_msg = "Creating lobby...".to_string();
                }
            }
            state.menu = MenuState::Lobby;
            Task::none()
        }
        AppMessage::Connect => {
            if !state.connected {
                let ws_tx = Arc::clone(&state.ws_tx);
                let server_rx = Arc::clone(&state.server_rx);
                let ip = state.ip.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(connect_ws(ws_tx, server_rx, ip));
                });
                state.msg = "Connecting...".into();
                state.connecting = true;
            }
            Task::none()
        }

        AppMessage::ToggleReady(player_to_toggle) => {
            // Finds the player and toggles their ready state.
            let new_ready_state = if let Some(lobby) = &state.lobby {
                if let Some(our_player) = lobby
                    .players
                    .iter()
                    .find(|player| player.id == player_to_toggle)
                {
                    !our_player.ready
                } else {
                    false
                }
            } else {
                false
            };

            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetReady {
                        ready: new_ready_state,
                    });
                    state.last_msg = format!("Set ready: {new_ready_state}");
                }
            }
            Task::none()
        }
        AppMessage::StartGame => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::StartGame);
                    state.last_msg = "Starting game...".to_string();
                }
            }
            Task::none()
        }

        AppMessage::GameRules => {
            state.menu = MenuState::Rules;
            Task::none()
        }
        AppMessage::BackToMenu => {
            // Stops the server if the player is the host; otherwise drops the connection.
            let am_host = if let (Some(lobby), Some(my_id)) = (&state.lobby, state.my_id) {
                lobby.players.iter().any(|p| p.id == my_id && p.is_host)
            } else {
                false
            };

            if am_host {
                if let Ok(guard) = state.ws_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(C::RequestShutdown);
                    }
                }
            }

            if let Ok(mut guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::LeaveLobby);
                }
                *guard = None;
            }
            if let Ok(mut guard_rx) = state.server_rx.lock() {
                *guard_rx = None;
            }

            state.connected = false;
            state.connecting = false;
            state.my_id = None;
            state.lobby = Some(Lobby {
                players: Vec::new(),
                chat: Vec::new(),
            });
            state.chat_input.clear();
            state.server_messages.clear();
            state.last_msg.clear();
            state.join_name.clear();
            state.host_name.clear();
            state.host_player_count = PlayerCount::P4;
            state.menu = MenuState::Main;
            Task::none()
        }
        AppMessage::CloseGame => {
            // Calls the Application exit.
            std::process::exit(0);
        }

        AppMessage::Tick => {
            handle_tick(state);
            Task::none()
        }
    }
}

fn handle_tick(state: &mut App) {
    if state.connecting && !state.connected {
        state.last_msg = "Connecting".to_string();
        if let Ok(guard) = state.ws_tx.lock() {
            if guard.is_some() {
                state.connected = true;
                state.connecting = false;
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::JoinLobby {
                        name: state.join_name.clone(),
                    });
                    state.last_msg = "Joining lobby...".to_string();
                    state.menu = MenuState::Lobby;
                }
            }
        }
    }
    // Collects messages first to avoid borrowing issues.
    let messages: Vec<ServerMessage> = if let Ok(guard) = state.server_rx.lock() {
        if let Some(ref rx) = *guard {
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                println!("Received: {msg:?}");
                msgs.push(msg);
            }
            msgs
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    for msg in messages {
        handle_server_message(state, msg);
    }
}

fn handle_server_message(state: &mut App, msg: ServerMessage) {
    match msg {
        ServerMessage::Server(s) => match s {
            S::HandshakeConfirmation { version, supported } => {
                state.last_msg = format!("Handshake: version {version}, supported {supported}");
            }
            S::JoinConfirmation { ok, id } => {
                state.last_msg = format!("Join: {ok}, id: {id}");
                state.my_id = Some(id);
            }
            S::Error { reason } => {
                state.last_msg = format!("Error: {reason}");
            }
            S::HandDealt { cards } => {
                state.last_msg = format!("Hand dealt: {} cards", cards.len());
            }
            S::BidRequest { min, max } => {
                state.last_msg = format!("Bid request: {min}-{max}");
            }
            S::InvalidBid { reason } => {
                state.last_msg = format!("Invalid bid: {reason}");
            }
            S::YourTurn { valid_cards } => {
                state.last_msg = format!("Your turn: {} cards", valid_cards.len());
            }
            S::InvalidMove { reason } => {
                state.last_msg = format!("Invalid move: {reason}");
            }
        },
        ServerMessage::Broadcast(b) => match b {
            B::LobbyState { lobby } => {
                state.last_msg =
                    format!("Lobby: {} players", lobby.as_ref().unwrap().players.len());
                state.lobby = lobby;
            }
            B::PlayerCountChanged { count } => {
                state.last_msg = format!("Max players set to {count}");

                // Converts the usize to a `PlayerCount` enum.
                // Needed for easier view handling.
                state.host_player_count = match count {
                    3 => PlayerCount::P3,
                    4 => PlayerCount::P4,
                    5 => PlayerCount::P5,
                    6 => PlayerCount::P6,
                    _ => PlayerCount::P4,
                };
            }
            B::ChatMessage { sender, message } => {
                state.last_msg = format!("Chat from {sender}: {message}");
                state.server_messages.push(format!("{sender}: {message}"));
                if let Some(ref mut lobby) = state.lobby {
                    lobby.chat.push((sender, message));
                }
            }
            B::GameStarted { players } => {
                state.last_msg = format!("Game started: {} players", players.len());
                state.menu = MenuState::Playing;
            }
            B::RoundStarted {
                round,
                cards_per_player,
                trump: _,
            } => {
                state.last_msg = format!("Round {round} started: {cards_per_player} cards");
            }
            B::BiddingStarted {
                starting_player: _,
                cards_per_player,
            } => {
                state.last_msg = format!("Bidding started: {cards_per_player} cards");
            }
            B::BidTurn { player } => {
                state.last_msg = format!("Player {player} bidding");
            }
            B::BidMade { player, amount } => {
                state.last_msg = format!("Player {player} bid {amount}");
            }
            B::BiddingFinished { bids } => {
                state.last_msg = format!("Bidding done: {} bids", bids.len());
            }
            B::PoolStarted { leader } => {
                state.last_msg = format!("Pool started, leader: {leader}");
            }
            B::TurnChanged { player } => {
                state.last_msg = format!("Turn: player {player}");
            }
            B::CardPlayed { player, card: _ } => {
                state.last_msg = format!("Player {player} played card");
            }
            B::PoolFinished { winner, cards: _ } => {
                state.last_msg = format!("Pool won by {winner}");
            }
            B::RoundFinished {
                scores: _,
                won_amounts: _,
            } => {
                state.last_msg = "Round finished".to_string();
            }
            B::GameFinished {
                final_scores: _,
                winner,
            } => {
                state.last_msg = format!("Game won by {winner}");
            }
            B::ServerShutdown => {
                println!("Client: received ServerShutdown broadcast");
                // Performs a cleanup (although maybe we can make a function for that)
                state.last_msg = "Lost connection to host".to_string();
                state.menu = MenuState::Main;
                state.connected = false;
                state.connecting = false;
                state.my_id = None;
                state.lobby = Some(Lobby {
                    players: Vec::new(),
                    chat: Vec::new(),
                });
                state.chat_input.clear();
                state.server_messages.clear();
            }
        },
    }
}
