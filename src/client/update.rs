use iced::Task;
use std::sync::Arc;

use super::{connect_ws, App, AppMessage, MenuState, PlayerCount};
use crate::api::{Card, Lobby, PlayerId, ServerMessage, Value, B, C, S};
use crate::client::TaskBatcher;
use crate::gameplay_ui::hand::hand_card::CardMessage;
use crate::gameplay_ui::hand::HandMessage;
use crate::gameplay_ui::scoreboard::ScoreBoardMessage;
use crate::gameplay_ui::table::middle::card_deck::CardDeckMessage;
use crate::gameplay_ui::table::TableMessage;
use crate::gameplay_ui::GameViewMessage;
use crate::ui_element_traits::{Animated, Message, Notifiable, Resizable};

/// Get player name from ID using lobby data
fn get_player_name(state: &App, player_id: PlayerId) -> String {
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

/// Format a card for display (e.g., "5 Red", "Wizard", "Jester")
fn format_card(card: &Card) -> String {
    let value_str = match card.value {
        Value::Jester => "Jester".to_string(),
        Value::Wizard => "Wizard".to_string(),
        Value::Number(n) => n.to_string(),
    };

    match card.value {
        Value::Jester | Value::Wizard => value_str,
        Value::Number(_) => format!("{} {:?}", value_str, card.suit),
    }
}

fn is_msg_not_ready(state: &App, msg: AppMessage) -> bool {
    if state.animation_count_down_latch > 0 {
        match msg {
            AppMessage::GameViewMessage(GameViewMessage::NewRound(_, _, _)) => true,
            AppMessage::GameViewMessage(GameViewMessage::ChangeTurn(_, _)) => true,
            AppMessage::GameViewMessage(GameViewMessage::NewTrick) => true,
            AppMessage::GameViewMessage(GameViewMessage::ScoreBoardMessage(
                ScoreBoardMessage::Update(_),
            )) => true,
            _ => false,
        }
    } else {
        false
    }
}

pub fn update(state: &mut App, msg_unaltered: AppMessage) -> Task<AppMessage> {
    match msg_unaltered.clone() {
        AppMessage::AnimationTick => (),
        AppMessage::ServerTick => (),
        AppMessage::GameViewMessage(GameViewMessage::ScoreBoardMessage(
            ScoreBoardMessage::Update(_),
        )) => (),
        /* AppMessage::GameViewMessage(GameViewMessage::HandMessage(HandMessage::CardMessage(
            CardMessage::Hovered(_),
        ))) => (),
        AppMessage::GameViewMessage(GameViewMessage::HandMessage(HandMessage::CardMessage(
            CardMessage::NotHovered(_),
        ))) => (), */
        AppMessage::GameViewMessage(GameViewMessage::HandMessage(HandMessage::CardMessage(
            CardMessage::CursorMoved(_, _),
        ))) => (),
        _ => {
            println!("{:?}", msg_unaltered)
        }
    };
    let mut tb = TaskBatcher::new();
    for queue_msg in state.msg_queue.iter() {
        tb.push_msg(queue_msg.clone())
    }
    state.msg_queue.clear();
    if is_msg_not_ready(state, msg_unaltered.clone()) {
        state.msg_queue_delayed.push(msg_unaltered);
        return tb.batch();
    }
    let msg = match msg_unaltered.clone() {
        AppMessage::GameViewMessage(GameViewMessage::ScoreBoardMessage(
            ScoreBoardMessage::Update(_),
        )) => AppMessage::GameViewMessage(GameViewMessage::ScoreBoardMessage(
            ScoreBoardMessage::Update(state.scoreboard_info()),
        )),
        _ => msg_unaltered,
    };
    match msg {
        AppMessage::DecrementACDL(amount) => {
            if state.animation_count_down_latch >= amount {
                state.animation_count_down_latch -= amount;
            } else {
                state.animation_count_down_latch = 0;
            }
            if state.animation_count_down_latch == 0 {
                for queue_msg in state.msg_queue_delayed.iter() {
                    tb.push_msg(queue_msg.clone())
                }
                state.msg_queue_delayed.clear();
            }
        }
        AppMessage::IncrementACDL(amount) => state.animation_count_down_latch += amount,
        AppMessage::Navigate(menu) => {
            state.menu = menu;
        }
        AppMessage::Host => {
            let local_ip = crate::server::local_ip();
            state.msg = format!("Hosting on {local_ip}");
            state.ip = local_ip;

            state.menu = MenuState::Host;
        }
        AppMessage::HostNameChanged(name) => {
            state.host_name = name;
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
        }
        AppMessage::JoinNameChanged(name) => {
            state.join_name = name;
        }
        AppMessage::ServerAddressChanged(addr) => {
            state.ip = addr;
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
        }
        AppMessage::ChatInputChanged(input) => {
            state.chat_input = input;
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
                    let _ = tx.send(C::SetPlayerCount {
                        count: state.host_player_count.to_usize(),
                    });
                    let _ = tx.send(C::JoinLobby {
                        name: state.host_name.clone(),
                    });
                    state.last_msg = "Creating lobby...".to_string();
                }
            }
            state.menu = MenuState::Lobby;
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
        }
        AppMessage::StartGame => {
            let can_start = if cfg!(feature = "wiz_debug") {
                true
            } else {
                state.lobby.as_ref().is_some_and(|lobby| {
                    lobby.players.len() == state.host_player_count.to_usize()
                        && lobby.players.iter().all(|p| p.ready)
                })
            };
            let is_host = state.my_id.is_some()
                && state.my_id.unwrap()
                    == state
                        .lobby
                        .as_ref()
                        .and_then(|l| l.players.iter().find(|p| p.is_host).map(|p| p.id))
                        .unwrap_or_default();
            if can_start && is_host {
                if let Ok(guard) = state.ws_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(C::StartGame);
                        state.last_msg = "Starting game...".to_string();
                    }
                }
            }
        }

        AppMessage::GameRules => {
            state.menu = MenuState::Rules;
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
            // Reset gameplay state
            state.game_log.clear();
            state.hand.clear();
            state.current_trick.clear();
            state.trump = None;
            state.round_number = 0;
            state.is_my_turn = false;
            state.is_bidding_phase = false;
            state.must_set_trump = false;
            state.current_player = None;
            state.player_order.clear();
            state.bids.clear();
            state.tricks_won.clear();
            state.scores.clear();
            state.bid_input.clear();
            state.valid_cards.clear();
            state.dealer = None;
            state.game_over = false;
            state.winner = None;
            state.menu = MenuState::Main;
        }
        AppMessage::CloseGame => {
            // Calls the application to exit.
            std::process::exit(0);
        }

        // Gameplay message handlers
        AppMessage::BidInputChanged(input) => {
            state.bid_input = input;
        }
        AppMessage::SubmitBid => {
            if let Ok(amount) = state.bid_input.parse::<usize>() {
                if let Ok(guard) = state.ws_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(C::Bid { amount });
                        let log = format!("[YOU] Submitting bid: {}", amount);
                        println!("{}", log);
                        state.game_log.push(log);
                        state.bid_input.clear();
                    }
                }
            } else {
                state.last_msg = "Invalid bid - enter a number".to_string();
            }
        }
        AppMessage::PlayCard(card) => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::PlayCard { card });
                    let log = format!("[YOU] Playing card: {:?} of {:?}", card.value, card.suit);
                    println!("{}", log);
                    state.game_log.push(log);
                }
            }
        }
        AppMessage::SetTrump(suit) => {
            if let Ok(guard) = state.ws_tx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(C::SetTrump { suit });
                    let log = format!("[YOU] Setting trump to: {:?}", suit);
                    println!("{}", log);
                    state.game_log.push(log);
                    state.must_set_trump = false;
                }
            }
        }
        AppMessage::ServerTick => {
            handle_tick(state);
            tb.push_msg(ScoreBoardMessage::Update(state.scoreboard_info()));
        }
        AppMessage::GameViewMessage(game_view_msg) => {
            tb.push(state.game_view.update_with_msg(game_view_msg));
        }
        AppMessage::ButtonMessage(btn_msg) => {
            // Route to buttons (each button filters by id internally)
            tb.push_mult([
                state.btn_host.update_with_msg(btn_msg.clone()),
                state.btn_join.update_with_msg(btn_msg.clone()),
                state.btn_rules.update_with_msg(btn_msg.clone()),
                state.btn_close.update_with_msg(btn_msg.clone()),
                state.btn_create_lobby.update_with_msg(btn_msg.clone()),
                state.btn_back.update_with_msg(btn_msg.clone()),
                state.btn_connect.update_with_msg(btn_msg.clone()),
                state.btn_send_chat.update_with_msg(btn_msg.clone()),
                state.btn_start_game.update_with_msg(btn_msg.clone()),
                state.btn_back_to_menu.update_with_msg(btn_msg.clone()),
                state.btn_ready_owned.update_with_msg(btn_msg.clone()),
                state.game_view.update_buttons_with_msg(btn_msg.clone()),
            ]);
        }
        AppMessage::AnimationTick => {
            tb.push_mult([
                state.game_view.update_animations(),
                // Update button animations
                state.btn_host.update_animations(),
                state.btn_join.update_animations(),
                state.btn_rules.update_animations(),
                state.btn_close.update_animations(),
                state.btn_create_lobby.update_animations(),
                state.btn_back.update_animations(),
                state.btn_connect.update_animations(),
                state.btn_send_chat.update_animations(),
                state.btn_start_game.update_animations(),
                state.btn_back_to_menu.update_animations(),
                state.btn_ready_owned.update_animations(),
            ]);
        }
        AppMessage::WindowResized(window_size) => {
            state.window_size = window_size;
            state.game_view.update_size(window_size);
        }
    }
    tb.batch()
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
    // Detect if the connection was dropped externally (ws_tx went None while connected).
    let mut messages: Vec<ServerMessage> = if state.connected {
        if let Ok(guard) = state.ws_tx.lock() {
            if guard.is_none() {
                vec![ServerMessage::ConnectionClosed]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Collects messages first to avoid borrowing issues.
    if let Ok(guard) = state.server_rx.lock() {
        if let Some(ref rx) = *guard {
            while let Ok(msg) = rx.try_recv() {
                println!("Received: {msg:?}");
                messages.push(msg);
            }
        }
    };

    for msg in messages {
        handle_server_message(state, msg);
    }
}

fn handle_server_message(state: &mut App, msg: ServerMessage) {
    match msg {
        ServerMessage::ConnectionClosed => {
            println!("Connection lost, resetting state.");
            state.connected = false;
            state.connecting = false;
            state.disconnected = true;
            state.my_id = None;
            state.lobby = Some(Lobby {
                players: Vec::new(),
                chat: Vec::new(),
            });
            state.chat_input.clear();
            state.server_messages.clear();
            state.last_msg = "Disconnected from server.".to_string();
            state.game_log.clear();
            state.hand.clear();
            state.current_trick.clear();
            state.trump = None;
            state.round_number = 0;
            state.is_my_turn = false;
            state.is_bidding_phase = false;
            state.must_set_trump = false;
            state.current_player = None;
            state.player_order.clear();
            state.bids.clear();
            state.tricks_won.clear();
            state.scores.clear();
            state.bid_input.clear();
            state.valid_cards.clear();
            state.dealer = None;
            state.game_over = false;
            state.winner = None;
            state.menu = MenuState::Main;
        }
        ServerMessage::Server(s) => match s {
            S::HandshakeConfirmation { version, supported } => {
                let log = format!("[SERVER] Handshake: version {version}, supported {supported}");
                println!("{}", log);
                state.last_msg = log.clone();
            }
            S::JoinConfirmation { ok, id } => {
                let log = format!("[SERVER] Join confirmed: ok={ok}, your_id={id}");
                println!("{}", log);
                state.last_msg = log;
                state.my_id = Some(id);
                state
                    .btn_ready_owned
                    .set_on_click(AppMessage::ToggleReady(id));
                state.disconnected = false;
            }
            S::Error { reason } => {
                let log = format!("[ERROR] {reason}");
                println!("{}", log);
                state.last_msg = log.clone();
                state.game_log.push(log);
            }
            S::HandDealt { cards } => {
                let log = format!("[SERVER] Hand dealt: {} cards", cards.len());
                println!("{}", log);
                for card in &cards {
                    println!("  - {:?} of {:?}", card.value, card.suit);
                }
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.hand = cards.clone();

                state
                    .msg_queue
                    .push(GameViewMessage::NewRound(state.trump, cards, Vec::new()).convert_msg());
            }
            S::TrumpRequest => {
                let log = "[SERVER] You must set the trump suit!".to_string();
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.must_set_trump = true;
                state.is_my_turn = true;
            }
            S::BidRequest { min, max } => {
                let log = format!("[SERVER] Your turn to bid! (range: {min}-{max})");
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.is_my_turn = true;
                state.is_bidding_phase = true;
            }
            S::InvalidBid { reason } => {
                let log = format!("[ERROR] Invalid bid: {reason}");
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
            }
            S::YourTurn { valid_cards } => {
                let log = format!(
                    "[SERVER] Your turn to play! {} valid cards",
                    valid_cards.len()
                );
                println!("{}", log);
                for card in &valid_cards {
                    println!("  - {:?} of {:?}", card.value, card.suit);
                }
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.is_my_turn = true;
                state.is_bidding_phase = false;
                state.valid_cards = valid_cards;
                state.msg_queue.push(
                    GameViewMessage::ChangeTurn(state.my_id.unwrap(), state.valid_cards.clone())
                        .convert_msg(),
                );
            }
            S::InvalidMove { reason } => {
                let log = format!("[ERROR] Invalid move: {reason}");
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
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
                state.player_order = players.clone();
                let player_names: Vec<String> = players
                    .iter()
                    .map(|id| get_player_name(state, *id))
                    .collect();
                let log = format!(
                    "[GAME] Game started with {} players: {}",
                    players.len(),
                    player_names.join(", ")
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;

                // Check if the host's name is "wizard_master" to enter debug
                // Easter egg :)
                if let Some(ref lobby) = state.lobby {
                    if let Some(host) = lobby.players.iter().find(|p| p.is_host) {
                        if host.name == "wizard_master" {
                            state.menu = MenuState::PlayingTest;
                        } else {
                            state.menu = MenuState::Playing;
                        }
                    }
                }

                state.scores.clear();
                state.bids.clear();
                state.tricks_won.clear();
                state
                    .msg_queue
                    .push(GameViewMessage::StartGame(state.game_start_info()).convert_msg());
            }
            B::RoundStarted {
                round,
                cards_per_player,
                trump,
            } => {
                let log = format!(
                    "[ROUND] Round {} started: {} cards, trump: {:?}",
                    round, cards_per_player, trump
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.round_number = round;
                state.trump = trump;
                state.current_trick.clear();
                state.bids.clear();
                state.tricks_won.clear();
                state.is_bidding_phase = true;
                // Reset turn state - only the player who receives BidRequest should be able to bid
                state.is_my_turn = false;
                state.must_set_trump = false;
                state.valid_cards.clear();
            }
            B::DealerMustSetTrump { dealer } => {
                let is_me = state.my_id == Some(dealer);
                let dealer_name = get_player_name(state, dealer);
                let log = format!(
                    "[ROUND] {} must set trump (Wizard drawn){}",
                    dealer_name,
                    if is_me { " - THAT'S YOU!" } else { "" }
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.dealer = Some(dealer);
                if is_me {
                    state.must_set_trump = true;
                }
                state
                    .msg_queue
                    .push(TableMessage::ChangeTurn(dealer).convert_msg());
            }
            B::TrumpSet { suit, by_dealer } => {
                let dealer_name = get_player_name(state, by_dealer);
                let log = format!("[ROUND] Trump set to {:?} by {}", suit, dealer_name);
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.trump = Some(Card {
                    suit,
                    value: Value::Number(1),
                });
                state.must_set_trump = false;
                state
                    .msg_queue
                    .push(CardDeckMessage::ChangeGlow(state.trump.unwrap()).convert_msg())
            }
            B::BiddingStarted {
                starting_player,
                cards_per_player,
            } => {
                let player_name = get_player_name(state, starting_player);
                let log = format!(
                    "[BIDDING] Bidding started: {} cards, starting with {}",
                    cards_per_player, player_name
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.is_bidding_phase = true;
                state.current_player = Some(starting_player);
            }
            B::BidTurn { player } => {
                let is_me = state.my_id == Some(player);
                let player_name = get_player_name(state, player);
                let log = format!(
                    "[BIDDING] {}'s turn to bid{}",
                    player_name,
                    if is_me { " - YOUR TURN!" } else { "" }
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.current_player = Some(player);
                state
                    .msg_queue
                    .push(TableMessage::ChangeTurn(player).convert_msg());
            }
            B::BidMade { player, amount } => {
                let player_name = get_player_name(state, player);
                let log = format!("[BIDDING] {} bid {}", player_name, amount);
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.bids.insert(player, amount);
                state.is_my_turn = false;
            }
            B::BiddingFinished { bids } => {
                let bids_with_names: Vec<String> = bids
                    .iter()
                    .map(|(id, amount)| format!("{}: {}", get_player_name(state, *id), amount))
                    .collect();
                let log = format!("[BIDDING] Bidding complete: {}", bids_with_names.join(", "));
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.bids = bids.into_iter().collect();
                state.is_bidding_phase = false;
                state.is_my_turn = false;
            }
            B::PoolStarted { leader } => {
                let is_me = state.my_id == Some(leader);
                let leader_name = get_player_name(state, leader);
                let log = format!(
                    "[TRICK] New trick started, leader: {}{}",
                    leader_name,
                    if is_me { " - YOUR LEAD!" } else { "" }
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.current_trick.clear();
                state.current_player = Some(leader);
                // Reset turn - only the leader who receives YourTurn should be able to play
                state.is_my_turn = false;
                state.valid_cards.clear();
                state
                    .msg_queue
                    .push(GameViewMessage::NewTrick.convert_msg());
            }
            B::TurnChanged { player } => {
                let is_me = state.my_id == Some(player);
                let player_name = get_player_name(state, player);
                let log = format!(
                    "[TRICK] Turn changed to {}{}",
                    player_name,
                    if is_me { " - YOUR TURN!" } else { "" }
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.current_player = Some(player);
                println!("~~~TURN CHANGE~~~");
                if !is_me {
                    state
                        .msg_queue
                        .push(GameViewMessage::ChangeTurn(player, Vec::new()).convert_msg());
                }
            }
            B::CardPlayed { player, card } => {
                let player_name = get_player_name(state, player);
                let card_str = format_card(&card);
                let log = format!("[TRICK] {} played {}", player_name, card_str);
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                state.current_trick.push((player, card));
                state.is_my_turn = false;
                // Remove card from hand if it was ours
                if state.my_id == Some(player) {
                    state.hand.retain(|c| *c != card);
                }
                state
                    .msg_queue
                    .push(GameViewMessage::CardPlayed(player, card).convert_msg());
                state.msg_queue.push(AppMessage::IncrementACDL(1));
            }
            B::PoolFinished { winner, cards } => {
                let is_me = state.my_id == Some(winner);
                let winner_name = get_player_name(state, winner);
                let log = format!(
                    "[TRICK] Trick won by {}{}",
                    winner_name,
                    if is_me { " - YOU WON!" } else { "" }
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                *state.tricks_won.entry(winner).or_insert(0) += 1;
                state.current_trick = cards;
            }
            B::RoundFinished {
                scores,
                won_amounts,
            } => {
                let scores_with_names: Vec<String> = scores
                    .iter()
                    .map(|(id, score)| format!("{}: {}", get_player_name(state, *id), score))
                    .collect();
                let tricks_with_names: Vec<String> = won_amounts
                    .iter()
                    .map(|(id, won)| format!("{}: {}", get_player_name(state, *id), won))
                    .collect();
                let log = format!(
                    "[ROUND] Round finished! Scores: {} | Tricks won: {}",
                    scores_with_names.join(", "),
                    tricks_with_names.join(", ")
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                for (player, score) in scores {
                    println!("[DEBUG] Inserting score for player {}: {}", player, score);
                    state.scores.insert(player, score);
                }
                println!("[DEBUG] state.scores after update: {:?}", state.scores);
            }
            B::GameFinished {
                final_scores,
                winner,
            } => {
                let is_me = state.my_id == Some(winner);
                let winner_name = get_player_name(state, winner);
                let scores_with_names: Vec<String> = final_scores
                    .iter()
                    .map(|(id, score)| format!("{}: {}", get_player_name(state, *id), score))
                    .collect();
                let log = format!(
                    "[GAME] GAME OVER! Winner: {}{} Final scores: {}",
                    winner_name,
                    if is_me { " - YOU WON!" } else { "" },
                    scores_with_names.join(", ")
                );
                println!("{}", log);
                state.game_log.push(log.clone());
                state.last_msg = log;
                // Store final scores and mark game as over
                for (player, score) in final_scores {
                    state.scores.insert(player, score);
                }
                state.game_over = true;
                state.winner = Some(winner);
                state
                    .msg_queue
                    .push(GameViewMessage::EndGame(winner).convert_msg());
            }
            B::ServerShutdown => {
                println!("[SERVER] Server shutdown received");
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
