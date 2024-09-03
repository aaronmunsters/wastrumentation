#!/usr/bin/env bash

### 1. Copy the dynamic analysis input programs to the working directory
rm -rf ./working-directory/{pure-functions-profiler,wastrumentation-rs-stdlib}
cp -r input-analyses/{pure-functions-profiler,wastrumentation-rs-stdlib} ./working-directory/

### 2. Use results from static analysis to build dynamic analysis
node    ./inject-static-analysis-results-into-dynamic.js        \
        ./working-directory/pure-function-indices.json          \
        ./working-directory/pure-functions-profiler/src/lib.rs  \
      > ./working-directory/pure-functions-profiler/src/lib_generated.rs

mv ./working-directory/pure-functions-profiler/src/lib_generated.rs \
   ./working-directory/pure-functions-profiler/src/lib.rs

# ###
# ### 2. Instrument
# ###
# TODO: move to use name in ./working-directory/target_wasm_program_name.txt
input_program_path=$(realpath ./working-directory/*.wasm)
rust_path=$(realpath ./working-directory/pure-functions-profiler/Cargo.toml)
output_path="./working-directory/instrumented.wasm"

# Turn JSON functions `[1,2,3]` into space-separated targets `1 2 3`
target_functions=`cat ./working-directory/pure-function-indices.json | sed -E 's/(\[|\]|\,)/ /g'`
target_functions_count=`echo ${target_functions} | wc -w`

cargo run --package wastrumentation-cli -- \
    --input-program-path ${input_program_path} \
    --rust-analysis-toml-path ${rust_path} \
    --hooks generic-apply           \
    --targets ${target_functions} \
    --output-path ${output_path}

cp ${input_program_path} `# ==SAVE-UNINSTRUMENTED-VARIANT==>` ./working-directory/uninstrumented.wasm
mv ${output_path} ${input_program_path}

# ###
# ### 3. Copy over analysis & harness
# ###

target_wasm_program_name=$(cat ./working-directory/target_wasm_program_name.txt)
bound_library_name=$(basename ${target_wasm_program_name} _bg)
bound_library="./${bound_library_name}.js"

cd working-directory
cp ../call_benchmark_instr.js .

# ###
# ### 4. Execute
# ###

node --experimental-wasm-multi-memory   \
    call_benchmark_instr.js             \
    ${bound_library}                    \
    ${target_functions_count}           \
    ./pure-function-indices.json        \
    called-pure-function-indices.json

# Clean up harness
rm ./call_benchmark_instr.js

cd ..
