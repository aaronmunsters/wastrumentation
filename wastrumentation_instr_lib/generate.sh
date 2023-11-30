# Code-gen the AssemblyScript library
mkdir -p   \
    dist/  \
    src_generated/

# check if modules installed
if [[ ! -e node_modules ]]; then
    npm i
fi

deno run --allow-write=./src_generated/ ./generate_for_signatures.ts

# Compile the AssemblyScript library to WebAssembly optimized
npx asc src_generated/lib.ts --textFile dist/wastrumentation_stack.wat -O3 \
    --disable bulk-memory \
    --runtime minimal \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json \
    --noExportMemory
npx asc src_generated/lib.ts -o dist/wastrumentation_stack.wasm -O3 \
    --disable bulk-memory \
    --runtime minimal \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json \
    --noExportMemory

# wasm-metadce dist/wastrumentation_stack.wasm --graph-file reachability.json -o dist/wastrumentation_stack.wasm
# wasm2wat dist/wastrumentation_stack.wasm -o dist/wastrumentation_stack.wat

# Compile the analysis
npx asc src_generated/analysis.ts --textFile dist/analysis.wat -O3 \
    --disable bulk-memory \
    --runtime stub \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json
npx asc src_generated/analysis.ts -o dist/analysis.wasm -O3 \
    --disable bulk-memory \
    --runtime stub \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json

# # DOCUMENTATION 
#        compilation unit             npx options used                       reason why
#        ----------------------------------------------------------------------------------------------
#        both                         (explicit) --config ...wasi-shim...    ensure that host functionality relies on Wasi, not JavaScript, cfr. [3]
#        both                         (explicit) --runtime minimal           the default runtime (GC) crashes after the binaryen merge pass, cfr. [2]
#        wastrumentation_stack        (explicit) --noExportMemory            used memory is not relevant to the outside
#        analysis                     (implicit) --ExportMemory              memory must be exposed for WASI to work, cfr. [1]
#
# ## TODOs
# - Short term: merge code dupe to have compilation configurations
# - Short term: take into account mergin _start
# - Long term: merge compilation & generation into instrumentation framework
#
# SRC:
# [1] https://github.com/bytecodealliance/wasmtime/issues/4985
# [2] Not sure what is the cause, but merging (ie. moving certain modules to own memory) crashes the default runtime
# [3] https://github.com/AssemblyScript/wasi-shim