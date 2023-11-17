#!/usr/bin/env bash

##########################################################################
################################## STEPS #################################
#### 1 . WAT -> WASM                                                 #####
#### 2 . WASM -> instrumented.wasm                                   #####
#### 3 . Generate library.wasm + analysis.wasm                       #####
#### 4 . Link [instrumented.wasm], [library.wasm], [analysis.wasm]   #####
#### 5 . Execute!                                                    #####
##########################################################################
##########################################################################

# Author: Aäron Munsters

#### 1 .
SCRIPT_INPUT_WAT_WASM=$1

if [[ $# != 1 ]]; then
    echo "Expected 1 argument, received $#."
    exit
fi

if [[ $SCRIPT_INPUT_WAT_WASM == *.wat ]]; then
    SCRIPT_INPUT_WAT=$SCRIPT_INPUT_WAT_WASM
    SCRIPT_INPUT=${SCRIPT_INPUT_WAT%.wat}
    wat2wasm $SCRIPT_INPUT_WAT -o "${SCRIPT_INPUT}.wasm"
    SCRIPT_INPUT_WASM="${SCRIPT_INPUT}.wasm"
fi

if [[ $SCRIPT_INPUT_WAT_WASM == *.wasm ]]; then
    SCRIPT_INPUT_WASM=$SCRIPT_INPUT_WAT_WASM
fi

if [[ $SCRIPT_INPUT_WASM == "" ]]
then
    echo "Provided input '${SCRIPT_INPUT_WAT_WASM}' is nor a .wat nor a .wasm file"
    exit
fi

SCRIPT_INPUT=${SCRIPT_INPUT_WASM%.wasm}
SCRIPT_INPUT_WASM_FULL_PATH=$(pwd)/${SCRIPT_INPUT_WASM}
SCRIPT_OUTPUT_INSTRUMENTED_WASM=${SCRIPT_INPUT}-instrumented.wasm
SCRIPT_OUTPUT_INSTRUMENTED_WASM_FULL_PATH=$(pwd)/${SCRIPT_OUTPUT_INSTRUMENTED_WASM}

#### 2 .
cargo run --package wastrumentation_cli -- \
    --input  ${SCRIPT_INPUT_WASM_FULL_PATH} \
    --output ${SCRIPT_OUTPUT_INSTRUMENTED_WASM_FULL_PATH}

#### 3 .
(
    cd wastrumentation_instr_lib/
    ./generate.sh
)

#### 4 .
wasm-merge -n --rename-export-conflicts --enable-multimemory --debuginfo --emit-text \
    ${SCRIPT_OUTPUT_INSTRUMENTED_WASM} instrumented_input \
    wastrumentation_instr_lib/dist/wastrumentation_stack.wasm wastrumentation_stack \
    wastrumentation_instr_lib/dist/analysis.wasm analysis \
    -o "merged.wat"
