WASP_DOT_WASP="example.wasp"
INPUT_DOT_WAT="add_two.wat"
INPUT_DOT_WASM="add_two.wasm"
OUT_DOT_WASM="add_two_instrumented.wasm"
OUT_DOT_WAT="add_two_instrumented.wat"

wat2wasm -o ${INPUT_DOT_WASM} ${INPUT_DOT_WAT}

#### 1 .
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
wasm-merge -n --rename-export-conflicts --enable-multimemory \
    ${OUT_DOT_WASM} instrumented_input \
    wastrumentation_instr_lib/dist/wastrumentation_stack.wasm wastrumentation_stack \
    wastrumentation_instr_lib/dist/analysis.wasm analysis \
    -o "merged.wasm"
