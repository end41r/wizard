#!/bin/bash

TARGET="linux"
DEST_PATH="/mnt/c/Users/capos/Downloads"
BUILD_TYPE="release"
FEATURES=""

while getopts "t:fh" opt; do
    case $opt in
        t)
            TARGET="$OPTARG"
            ;;
        f)
            FEATURES="wiz_debug $FEATURES"
            ;;
        h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  -t <target>    Build target (windows, linux, macos) [default: windows]"
            echo "  -f             Enable debug features"
            echo "  -h             Show help"
            exit 0
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

case $TARGET in
    windows)
        TARGET_NAME="x86_64-pc-windows-gnu"
        EXE_EXT=".exe"
        ;;
    linux)
        TARGET_NAME="x86_64-unknown-linux-gnu"
        EXE_EXT=""
        ;;
    macos)
        TARGET_NAME="aarch64-apple-darwin"
        EXE_EXT=""
        ;;
    *)
        echo "Unknown target: $TARGET"
        echo "Available targets: windows, linux, macos"
        exit 1
        ;;
esac

BUILD_CMD="cargo build --target $TARGET_NAME --release"
if [ -n "$FEATURES" ]; then
    BUILD_CMD="$BUILD_CMD --features $FEATURES"
fi


$BUILD_CMD && \
    cp "target/$TARGET_NAME/$BUILD_TYPE/wizard$EXE_EXT" "$DEST_PATH/" && \
    cp assets/ "$DEST_PATH/" -r && \