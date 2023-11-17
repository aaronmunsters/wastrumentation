# Code-gen the AssemblyScript library
mkdir -p   \
    dist/  \
    src/

# check if modules installed
if [[ ! -e node_modules ]]; then
    npm i
fi

deno run ./generate_for_signatures.ts > src/lib.ts
cp analysis.ts src/analysis.ts

# Compile the AssemblyScript library to WebAssembly optimized
npx asc src/lib.ts --textFile dist/wastrumentation_stack.wat -O \
    --runtime minimal \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json \
    --noExportMemory
npx asc src/lib.ts -o dist/wastrumentation_stack.wasm -O \
    --runtime minimal \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json \
    --noExportMemory

# Compile the analysis
npx asc src/analysis.ts --textFile dist/analysis.wat -O \
    --runtime minimal \
    --config ./node_modules/@assemblyscript/wasi-shim/asconfig.json
npx asc src/analysis.ts -o dist/analysis.wasm -O \
    --runtime minimal \
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