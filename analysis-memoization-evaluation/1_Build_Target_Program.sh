#!/usr/bin/env bash

rm -rf working-directory
mkdir working-directory

###
### 1. Compile rust program to wasm
###

wasm-pack build --target nodejs --out-dir ../../../working-directory ./input-programs/rust/boa-recursion
echo $(basename ./working-directory/*.wasm .wasm) > ./working-directory/target_wasm_program_name.txt
