rm -rf working-directory
mkdir working-directory

###
### 1. Compile wat program to wasm
###

wasm_source_code='
(module
    (func $add (param $a i32) (param $b i32) (result i32)
        (local.get $a)
        (local.get $b)
        (i32.add))
    (export "add" (func $add)))
'

cd working-directory

echo ${wasm_source_code} > add_program.wat
wat2wasm add_program.wat && rm add_program.wat

###
### 2. Instrument
###

input_program_path="./add_program.wasm"
rust_path=$(realpath "../../input-analyses/rust/call-stack-eq-wasabi/Cargo.toml")
output_path="./add_program_instrumented.wasm"

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
### 3. Copy over analysis
###
# Only required for Wasabi

###
### 4. Execute
###

js_source_code='
const wasm_bytes = require("fs").readFileSync("./add_program.wasm");
const wasm_module = new WebAssembly.Module(wasm_bytes);
const wasm_instance = new WebAssembly.Instance(wasm_module, {});
console.log(wasm_instance.exports.add(1, 2));

console.log({
    number_of_calls: wasm_instance.exports.get_number_of_calls(),
    max_call_depth: wasm_instance.exports.get_max_call_depth(),
    call_stack: wasm_instance.exports.get_call_stack(),
});
'
echo ${js_source_code} > node_execute_add_program.js
node --experimental-wasm-multi-memory node_execute_add_program.js
