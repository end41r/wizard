# wizard
A Wizard Card game ( https://en.wikipedia.org/wiki/Wizard_(card_game) ) based on the Iced UI Framework to play in a local network with friends


## To get the dependencies run 
`cargo build`

## Building for windows
Uncomment a line at the top of main to remove the stdout terminal for a better experience

## Running
`cargo run`

## Constraints
Please refer to REQUIREMENTS.md

We also have a pr check for clippy and fmt, so if it fails, please use
- `cargo fmt` to fix formatting (automatically)
- `cargo clippy` to check for the warnings (most of them will be gone if you use `cargo clippy --fix --bin "wizard"`)
- If you really trust clippy, you can also do `cargo clippy --fix --bin "wizard" --allow-dirty`, but its better not to use it :)
