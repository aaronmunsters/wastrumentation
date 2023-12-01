WASP_DOT_WASP="example.wasp"
INPUT_DOT_WAT="add_two.wat"
INPUT_DOT_WASM="add_two.wasm"
OUT_DOT_WASM="add_two_instrumented.wasm"
OUT_DOT_WAT="add_two_instrumented.wat"

wat2wasm -o ${INPUT_DOT_WASM} ${INPUT_DOT_WAT}

# #### 1 .
cargo run --package wastrumentation_cli -- \
    --wasp   ${WASP_DOT_WASP} \
    --input  ${INPUT_DOT_WASM} \
    --output ${OUT_DOT_WASM}
wasm2wat -o ${OUT_DOT_WAT} ${OUT_DOT_WASM}

#### 3 .
(
    cd wastrumentation_instr_lib/
    ./generate.sh
)

#### 4 .
cargo run --package patch-wasi-start-exports -- \
    --entry ${OUT_DOT_WASM} \
    --joins \
        wastrumentation_stack=wastrumentation_instr_lib/dist/wastrumentation_stack.wasm \
        WASP_ANALYSIS=wastrumentation_instr_lib/dist/analysis.wasm

wasm-merge --rename-export-conflicts --enable-multimemory \
    wastrumentation_instr_lib/dist/wastrumentation_stack.wasm   wastrumentation_stack   \
    wastrumentation_instr_lib/dist/analysis.wasm                WASP_ANALYSIS           \
    ${OUT_DOT_WASM}                                             instrumented_input      \
    -o "merged.wasm"

#### 5.
# RUST_LOG=trace wasmtime run --wasm multi-memory merged.wasm --invoke add-two 1 2
# wasm-interp --enable-multi-memory --wasi merged.wasm --trace --run-export "add-two" -a "i32:1" -a "i32:3" > trace.t

# wasmtime run \
#     --preload wastrumentation_stack=wastrumentation_instr_lib/dist/wastrumentation_stack.wasm \
#     --preload WASP_ANALYSIS=wastrumentation_instr_lib/dist/analysis.wasm \
#     ${OUT_DOT_WASM} --invoke add-two 1 2

cargo run --package execution-bench