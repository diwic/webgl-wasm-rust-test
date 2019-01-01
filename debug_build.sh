#!/bin/bash

# Quit on error
set -ex

cargo build --target wasm32-unknown-unknown

wasm-bindgen target/wasm32-unknown-unknown/debug/webgl_wasm.wasm --no-modules --out-dir target
cp public/index.html target/
