#!/usr/bin/env bash

target_functions=`cat ./working-directory/called-pure-function-indices-bash.txt`
target_functions_count=`echo ${target_functions} | wc -w`

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
