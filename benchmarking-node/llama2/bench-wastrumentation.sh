#!/usr/bin/env bash

mkdir -p working-dir && cd "$_" || exit

git_url_llama2_c="https://github.com/karpathy/llama2.c"
git_url_llama2_c_version="350e04fe35433e6d2941dce5a1f53308f87058eb"
folder_llama2_c="llama2.c"

#     ####################################################################
echo "##### 1. Git clone Llama2.c at specific hash if not cloned yet #####"
#     ####################################################################
if [[ ! -d ${folder_llama2_c} ]]; then
    mkdir ${folder_llama2_c}
    cd ${folder_llama2_c} || exit
    # src: https://stackoverflow.com/a/43136160
    git init --quiet && git remote add origin ${git_url_llama2_c}
    # clones +/- 2.2M of data
    git fetch --depth 1 origin ${git_url_llama2_c_version} --quiet
    git checkout FETCH_HEAD --quiet
else
    cd ${folder_llama2_c} || exit
    git checkout FETCH_HEAD --quiet
fi

#     ###############################
echo "##### 2. Patch the C code #####"
#     ###############################
# test if run.c has already changes applied (src: https://stackoverflow.com/a/25149786)
if [[ `git status run.c --porcelain` ]]; then git restore run.c; fi

cp ../../patch_for_benchmarks.patch .
git apply patch_for_benchmarks.patch
rm patch_for_benchmarks.patch

#     ############################
echo "##### 3. Fetch the LLM #####"
#     ############################

#   options: "15M" (58M of data), "42M" (159M of data), "110M" (418M of data)
model_bin_path="stories15M.bin"
wget -O ${model_bin_path} --no-clobber --no-verbose --quiet https://huggingface.co/karpathy/tinyllamas/resolve/main/${model_bin_path}

#     ##############################
echo "##### 4. Compile to wasm #####"
#     ##############################
llama_bin_name="llama2_c"
llama_js_bin_name=${llama_bin_name}.js # Output a NodeJS accessible script
emcc run.c \
    -o ${llama_js_bin_name} \
    -sALLOW_MEMORY_GROWTH                                         `# Allow memory to grow at runtime`                       \
    -O3                                                           `# Compiling for maximum perfomance`                      \
    -lm                                                           `# Linking the math library`                              \
    --preload-file ${model_bin_path} --preload-file tokenizer.bin `# Allow binary to access the llm model & the tokenizer`  \
    -s EXPORTED_FUNCTIONS=_inference                                                                                        \
    -s EXPORTED_RUNTIME_METHODS='["cwrap","FS","wasmExports"]'
cp ../../execute.js .

#     #######################
echo "##### 5. Execute! #####"
#     #######################

#     #############################
echo "##### W.1 Instrument! #####"
#     #############################
input_program_path=$(realpath ./${llama_bin_name}.wasm)
rust_path=$(realpath ../../../input-analyses/rust/call-stack-eq-wasabi/Cargo.toml)
output_path="./${llama_bin_name}_instrumented.wasm"
cargo run -- \
    --input-program-path ${input_program_path} \
    --rust-analysis-toml-path ${rust_path} \
    --hooks call-pre           \
            call-post          \
            call-indirect-pre  \
            call-indirect-post \
    --output-path ${output_path}
mv ${output_path} ${input_program_path}
node --experimental-wasm-multi-memory execute.js ${llama_js_bin_name} ${model_bin_path} # <-- optionally include `verbose` argument
