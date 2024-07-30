mkdir -p working-dir
cd working-dir

# TODO: pin these to a set version
git_url_llama2_c="https://github.com/karpathy/llama2.c"
git_url_wasi_libc="https://github.com/WebAssembly/wasi-libc.git"

folder_llama2_c="llama2.c"

# 1. Download Llama2.c
echo "Git clone Llama2.c if not cloned yet"
if [[ ! -d ${folder_llama2_c} ]]; then
    git clone "${git_url_llama2_c}" "${folder_llama2_c}"
else
    git -C "${folder_llama2_c}" pull
fi

# 2. Go to Llama2.c repository
cd ${folder_llama2_c}

# 3. Fetch the LLM
model_bin_path="model.bin"
wget -O ${model_bin_path} --no-clobber https://huggingface.co/karpathy/tinyllamas/resolve/main/stories15M.bin

# 4. Compile to wasm
llama_js_bin_name="llama2_c.js"
emcc run.c \
    -o ${llama_js_bin_name}                                       `# Output a NodeJS accessible script`  \
    -sALLOW_MEMORY_GROWTH                                         `# Allow memory to grow at runtime`    \
    -lm                                                           `# Linking the math library`           \
    -O3                                                           `# Compiling for maximum perfomance`   \
    --preload-file ${model_bin_path} --preload-file tokenizer.bin `# Allow binary to access the llm model & the tokenizer`

# Execute!
node ${llama_js_bin_name} ${model_bin_path}
