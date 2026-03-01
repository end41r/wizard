# DOCUMENTATION

## Overview

A digital implementation of the classic [Wizard card game](https://en.wikipedia.org/wiki/Wizard_(card_game)), built in Rust using the Iced UI framework for the client and Axum for the server, designed for playing in a local network.

---

## Building and Running

### Prerequisites

- **Cargo, Rust** toolchain (`edition=2024`, the project enforces `1.90.0` via `rust-toolchain.toml`)
- **On Linux:** the `alsa` library for audio must be installed (e.g. `sudo apt install libasound2-dev` on Ubuntu)

### Clone and Run

```sh
git clone https://github.com/end41r/wizard.git
cd wizard
cargo build
cargo run
```

### Cross-Compilation with `build.sh`

Primarily for WSL > Windows for better testing experience:

```sh
./build.sh -t windows    # Windows release build
./build.sh -t linux      # Linux release build
./build.sh -t windows -f # Windows release build with debug features (wiz_debug)
```

The script compiles a release binary and copies both the executable and the `assets/` folder to a directory specified in `DEST_PATH`.

### Running Tests

```sh
cargo test
```

Unit are located in `src/gamelogic/game.rs` (UI testing was done differently).

---

## Implemented Features and How to Discover Them

### Main Menu

When the application starts, the main menu is displayed with four buttons:

| Button | Action |
| -------- | -------- |
| **Host** | Opens the host menu to create a new game lobby |
| **Beitreten** | Opens the join menu to connect to an existing lobby |
| **Optionen** | Opens audio settings |
| **Spiel Verlassen** | Exits the application |

### Hosting a Game

1. Click **Host** in the main menu.
2. Enter your player name.
3. Select the player count (3–6 players).
4. Click **Lobby Erstellen** — this starts the WebSocket server on port `3000` and puts you in the lobby.
5. Share your IP address (shown in the lobby) with the other players.

### Joining a Game

1. Click **Beitreten** in the main menu.
2. Enter your player name and the host's IP address.
3. Click **Verbinden** to join the lobby.

### Lobby

- All connected players are listed with a **Bereit | Nicht Bereit** toggle.
- The lobby includes a chat where players can send messages.
- The game starts when all players are ready and the host presses **Starten**.

### Rules

The game follows the official Wizard card game rules. A full in-game rules screen is accessible via the **Spielregeln** button in **Optionen**. In brief:

#### Gameplay UI

- **Card hand:** Cards that are displayed at the bottom, with a right click you can also see which are playable.
- **Game table:** Shows other players' avatars, the current trick's cards in the middle, if you click the trick's cards you can toggle the view of all played cards in the trick, by just hovering you can temporarily see the played cards in the trick.
- **Scoreboard:** Displayed on the side, shows each player's current score, bid, and tricks won. The scoreboard also provides trump-suit selection buttons when the dealer must set trump.
- **Avatars:** Each player has an animated avatar (Elf, Knight, Mage, or Witch) with idle and casting animations. Casting is triggered when a player plays acard. Each avatar type has unique sound effects upon clicking the avatar.
- **Shards:** Float around the avatar of each player, indicating the amount of cards left in their hand.

### Audio

- **Background music** plays on the menu, in the lobby, and during gameplay (different tracks for each).
- **Sound effects** for: button clicks, card hover, card shuffle, card deal, card play, card error, shard play, avatar-specific casting and click sounds, and game over.
- **Volume control:** Music and SFX volume can be adjusted via sliders in the **Optionen** menu.

### Easter Egg

If the host enters the name `wizard_master`, the game launches in a **debug gameplay view** — a text-based, scrollable interface showing all game states (round, trump, bids, hand, tricks, scores, log) with minimal styling. This is located in `src/client/views/debug_gameplay.rs`.

### Debug Build Feature

Compiling with the `wiz_debug` Cargo feature (`cargo run --features wiz_debug`) enables:

- The Windows console window in release builds (for logging output).
- Better testing conditions (you can start with 1 player without waiting for everybody to get ready).

### Cross-Platform

The application runs on **Linux**, **Windows**, and **macOS**.

---

## Architecture

### Client ↔ Server Model

- `src/server.rs`: An Axum + Tokio WebSocket server. Manages player connections, lobby state, game state, and event broadcasting. Binds to `0.0.0.0:3000`.
- `src/client/`: An Iced GUI application. Handles user input, animations, rendering, audio playback, and communicates with the server via WebSockets.

### Core Modules

- `src/main.rs` : Entry point; launches the Iced client
- `src/api.rs` : Shared types, protocol messages (`C`, `S`, `B`), card/suit/value definitions, avatar types, lobby/player structs
- `src/server.rs` : Axum WebSocket server, game event dispatching, lobby management
- `src/gamelogic/` : Core game logic — `Game` (overall game flow) and `Round` (per-round state, bidding, trick resolution, scoring)
- `src/client/mod.rs` : Client application state (`App`), message enum, initialization
- `src/client/update.rs` : Message handling and state transitions
- `src/client/ws.rs` : WebSocket connection management
- `src/client/audio.rs` : Music and SFX playback via `rodio`
- `src/client/views/` : UI views: main menu, host menu, join menu, options, lobby, rules, debug gameplay
- `src/gameplay_ui/` : Game-table UI: hand display, card rendering, avatars, scoreboard, table layout
- `src/animation.rs` : Animation framework with easing functions (basic, auto-reversing, circular animations)
- `src/ui_element_traits.rs` : Shared traits for UI elements (Viewable, Animated, Resizable, etc.)

### Main Dependencies

- `iced` : UI framework (with image and tokio features)
- `axum` : Web server framework with WebSocket support
- `tokio` : Async runtime
- `tokio-tungstenite` : WebSocket client implementation
- `serde` / `serde_json` : Serialization and deserialization of protocol messages
- `rodio` : Audio playback (music and sound effects)

---

## Testing Method

### Approach

The project uses Unit tests for the core game logic module (`src/gamelogic/game.rs`). This is the only module we could write tests for.

### UI tests

- UI testing was done manually by running the application and verifying that all features work as intended through hours of playing together / alone, because iced doesnt provide a nice way to test it's elements.

---

## AI / LLM Usage

- **Easing functions** (`src/animation.rs`): Claude.ai generated the mathematical logic for easing functions (`ease_in_cubic`, `ease_out_cubic`, `ease_in_out_cubic`, `ease_in_sine`, `ease_out_sine`, `ease_in_out_sine`, `ease_out_elastic`, `ease_out_bounce`).
- **Animation macros** (`src/animation.rs`): Claude.ai helped to learn how to write Rust macros and partially generated the `impl_animation_common!` macro and trait bound patterns.
- **Scoreboard functions** (`src/gameplay_ui/scoreboard.rs`): AI helped to write the `sorted_player_order_by_score` function and the scoreboard view placement logic.
- **Pixel art rendering** (`src/gameplay_ui/table/avatar.rs`): Claude helped to learn how to use `filter_method` to achieve non-blurred pixel art scaling.
- **Hand generics** (`src/gameplay_ui/hand/mod.rs`): Claude.ai suggested passing a union type for generic hand card handling and helped to learn `Vec::contains` usage.
- **Server WebSocket handler** (`src/server.rs`): Claude Opus helped implement the WebSocket connection handler.

- **Array filtering** (`src/client/mod.rs`): Gemini helped to learn how to pass an array into a function and filter it.

### AI-Generated Images

The following image assets were created with AI assistance:

- `background_forall.png`, `ingame_background.png`, `menu_container.png`
- `wizard_lobby_menu.png`, `wizard_main_menu*.png`
- `button1.png`, `Menu_Button.png`
