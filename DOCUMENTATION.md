# DOCUMENTATION

## From README

A digital implementation of the classic [Wizard card game](https://en.wikipedia.org/wiki/Wizard_(card_game)), built in Rust using the Iced UI framework for the client and Axum for the server for playing in a local network.

---

## Architecture

### Client<->Server Model

- **Server**: Axum + Tokio, handles game state, player connections, and message dispatching via WebSockets.
- **Client**: Iced, user input, animations, rendering etc.

### Core Modules

- `src/api.rs`: Shared types, protocol messages, and serialization for client-server communication.
- `src/gamelogic/`: Game logic handling.
- `src/server.rs`: Axum-based WebSocket server, manages player sessions and event broadcasting.
- `src/client/`: UI & state management, audio, networking.
- `src/gameplay_ui/`: UI components for the game table, hand, avatars, and scoreboard.

### Audio System

- Uses `rodio` for music and sound effects.
- SFX and music are loaded from the `assets/audio/` directory.
- Allows music&sfx volume control.

---

## Features

- **Audio**: Background music and SFX for actions (e.g., card play, button clicks).
- **Debug Mode**: Enable with the `wiz_debug` feature to enable terminal on windows on release version.
- **Cross-Platform**: Runs on Linux, Windows, and MacOS.

### Easter Egg

- If the host sets their name to `wizard_master`, the game will be launched in debug mode with comprehensive logging.

---

## Building, Running

### Prerequisites

- Rust (edition `2021`) (backwards compatible with rust stable `2024`)
- Cargo
- For Linux You'd need to install `alsa`

### Installing

```sh
git clone https://github.com/end41r/wizard.git
cd wizard
cargo build
cargo run
```

### Building for WSL

```sh
./build.sh -t windows    # For Windows target
./build.sh -t linux      # For Linux target
./build.sh -t windows -f # For Windows target with debug features
```

- The script build a release version of the code and copies the build output to a shared folder (mostly for WSL-to-Windows transfer).

### Main Dependencies

- **iced**: UI framework
- **axum**: Web server and WebSocket support
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **rodio**: Audio playback
- **strum/strum_macros**: Enum utilities
