rm -rf working-dir
mkdir working-directory

###
### 1. Compile rust program to wasm
###

wasm-pack build --target nodejs --out-dir ../working-directory ./rust-boa-recursion

###
### 2. Instrument
###

cd working-directory
wasabi --node --hooks call rust_boa_recursion_bg.wasm
mv out/* . && rmdir out
cd ..

###
### 3. Copy over analysis & harness
###

cd working-directory
cp ../log-all.js .
cd ..

###
### 4. Execute
###

cd working-directory
cp ../node_execute_rust_boa_recursion.js .
node --experimental-wasm-multi-memory node_execute_rust_boa_recursion.js .
cd ..
