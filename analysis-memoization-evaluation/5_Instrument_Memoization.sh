#!/usr/bin/env bash

cp -r input-analyses/{pure-functions-memoization,wastrumentation-rs-stdlib-with-std} ./working-directory/
### 2. Use results from dynamic analysis to build memoization dynamic analysis
node    ./inject-dynamic-analysis-results-dynamic-memoization.js    \
        ./working-directory/called-pure-function-indices.json       \
        ./working-directory/pure-functions-memoization/src/lib.rs   \
        ./working-directory/called-pure-function-indices-bash.txt   \

# ###
# ### 2. Instrument
# ###

# the 'original' name of what the runtime expects
target_wasm_program_name=$(cat ./working-directory/target_wasm_program_name.txt)

input_program_path=$(realpath ./working-directory/uninstrumented.wasm)
rust_path=$(realpath ./working-directory/pure-functions-memoization/Cargo.toml)
output_path="./working-directory/${target_wasm_program_name}.wasm"

target_functions=`cat ./working-directory/called-pure-function-indices-bash.txt`
target_functions_count=`echo ${target_functions} | wc -w`
rm ${output_path}

cargo run --package wastrumentation-cli -- \
    --input-program-path ${input_program_path} \
    --rust-analysis-toml-path ${rust_path} \
    --hooks generic-apply           \
    --targets ${target_functions} \
    --output-path ${output_path}

# ###
# ### 3. Copy over analysis & harness
# ###

target_wasm_program_name=$(cat ./working-directory/target_wasm_program_name.txt)
bound_library_name=$(basename ${target_wasm_program_name} _bg)
bound_library="./${bound_library_name}.js"

cd working-directory
cp ../call_benchmark_memoization_instr.js .

# ###
# ### 4. Execute
# ###

node --experimental-wasm-multi-memory call_benchmark_memoization_instr.js ${bound_library} ${target_functions_count}
rm call_benchmark_memoization_instr.js
cd ..
