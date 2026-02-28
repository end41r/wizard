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
- `cargo clippy` to check for the warnings (most of them will be gone if you use `cargo clippy --fix --bin "wizard"` (commit your changes beforehand))
- If you really trust clippy, you can also do `cargo clippy --fix --bin "wizard" --allow-dirty`, but its better not to use it :)

### To run in debug mode with the wiz_debug feature enabled, run
`cargo run --features wiz_debug`
This will allow you to start the game with 1 player.

### build.sh usage
- Allows building release executable files for windows/linux/mac with `./build.sh -t <target>`, also allows to build with a `wiz_debug` feature flag with `./build.sh -t <target> -f`
- To build you would need to install targets first
- To correctly copy the assets, you would need to run the script from the root of the project
- The script copies the needed files as a release in your specified folder (in `DEST_PATH`)
