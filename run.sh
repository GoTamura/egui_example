#!/bin/env bash

set -e

echo "Compiling..."
cargo build --target wasm32-unknown-unknown

echo "Generating bindings..."
mkdir -p target/wasm-examples/egui_example
wasm-bindgen --target web --out-dir target/wasm-examples/egui_example target/wasm32-unknown-unknown/debug/egui_example.wasm
cp ../wgpu/wasm-resources/index.template.html target/wasm-examples/egui_example/index.html
sed -i "s/{{example}}/egui_example/g" target/wasm-examples/egui_example/index.html

# Find a serving tool to host the example
SERVE_CMD=""
SERVE_ARGS=""
if which basic-http-server; then
    SERVE_CMD="basic-http-server"
    SERVE_ARGS="target/wasm-examples/egui_example -a 127.0.0.1:1234"
elif which miniserve && python3 -m http.server --help > /dev/null; then
    SERVE_CMD="miniserve"
    SERVE_ARGS="target/wasm-examples/egui_example -p 1234 --index index.html"
elif python3 -m http.server --help > /dev/null; then
    SERVE_CMD="python3"
    SERVE_ARGS="-m http.server --directory target/wasm-examples/egui_example 1234"
fi

# Exit if we couldn't find a tool to serve the example with
if [ "$SERVE_CMD" = "" ]; then
    echo "Couldn't find a utility to use to serve the example web page. You can serve the `target/wasm-examples/egui_example` folder yourself using any simple static http file server."
fi

echo "Serving example with $SERVE_CMD at http://localhost:1234"
$SERVE_CMD $SERVE_ARGS
