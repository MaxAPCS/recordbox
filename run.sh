#!/usr/bin/bash

cargo build -p recordbox-ui --target wasm32-unknown-unknown -F console &&
~/.cargo/bin/wasm-bindgen --out-dir frontend/pkg --target web target/wasm32-unknown-unknown/debug/recordbox_ui.wasm &&
cargo run -p recordbox