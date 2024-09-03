#!/usr/bin/env bash

cd working-directory
cargo run --package wastrumentation-static-analysis -- \
    --input-program-path *.wasm \
    --output-path pure-function-indices.json \
    --minimum-body 1
