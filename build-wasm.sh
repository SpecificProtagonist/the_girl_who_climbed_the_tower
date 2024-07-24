#!/bin/sh
rm -r target/web
cp -r wasm target/web
cp -r assets target/web/
cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name bevy_game --out-dir target/web --target web target/wasm32-unknown-unknown/wasm-release/the_girl_who_climbed_the_tower.wasm
