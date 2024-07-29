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

rm -rf working-directory
mkdir working-directory
cd working-directory

echo ${wasm_source_code} > add_program.wat
wat2wasm add_program.wat && rm add_program.wat

###
### 2. Instrument
###

wasabi --node add_program.wasm
mv out/* . && rmdir out

###
### 3. Copy over analysis
###
cp ../log-all.js .

###
### 4. Execute
###

js_source_code='
globalThis.Wasabi = require("./add_program.wasabi.js");
const user_hooks = require("./log-all.js");

const wasm_bytes = require("fs").readFileSync("./add_program.wasm");
const wasm_module = new WebAssembly.Module(wasm_bytes);
const wasm_instance = new WebAssembly.Instance(wasm_module, {});
console.log(wasm_instance.exports.add(1, 2));
'
echo ${js_source_code} > node_execute_add_program.js
node node_execute_add_program.js
