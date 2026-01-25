#!/bin/bash

# Build the web version of the journal app

echo "Installing wasm32 target..."
rustup target add wasm32-unknown-unknown

echo "Installing trunk..."
cargo install --locked trunk

echo "Building web app..."
trunk build --release

echo ""
echo "Build complete! To run locally:"
echo "  trunk serve --open"
echo ""
echo "Or serve the dist/ folder with any static file server"
