rm -rf working-directory
mkdir working-directory

###
### 1. Compile rust program to wasm
###

wasm-pack build --target nodejs --out-dir ../working-directory ./rust-boa-recursion

###
### 2. Instrument
###

input_program_path=$(realpath ./working-directory/rust_boa_recursion_bg.wasm)
rust_path=$(realpath ../input-analyses/rust/call-stack-eq-wasabi/Cargo.toml)
output_path="./working-directory/rust_boa_recursion_bg_instrumented.wasm"

cargo run -- \
    --input-program-path ${input_program_path} \
    --rust-analysis-toml-path ${rust_path} \
    --hooks call-before          \
            call-after           \
            call-indirect-before \
            call-indirect-after  \
    --output-path ${output_path}

mv ${output_path} ${input_program_path}

###
### 3. Copy over analysis & harness
###

# Only required for Wasabi

###
### 4. Execute
###

cd working-directory
cp ../node_execute_rust_boa_recursion.js .
node --experimental-wasm-multi-memory node_execute_rust_boa_recursion.js .
cd ..
