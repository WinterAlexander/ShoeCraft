#!/bin/bash

# crate_name=cargo tree --depth 0 | awk '{print $1;}'
rm -rf ./out
cargo build --release --target wasm32-unknown-unknown
mkdir ./out
wasm-bindgen --out-dir ./out/ --target web --no-typescript --out-name "shoe_craft" target/wasm32-unknown-unknown/release/shoe_craft.wasm
cp index.html ./out
mkdir ./out/assets/
cp -r ./assets/* ./out/assets/
zip -rq out.zip out