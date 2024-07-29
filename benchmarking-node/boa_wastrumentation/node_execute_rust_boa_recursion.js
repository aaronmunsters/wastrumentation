// globalThis.Wasabi = require("./rust_boa_recursion_bg.wasabi.js");
// const user_hooks = require("./log-all.js");

const rust_boa_recursion = require("./rust_boa_recursion.js");

for (let i = 1; i <= 13; i++) {
    console.time(`compute_recursive_through_js(${i})`);
    rust_boa_recursion.compute_recursive_through_js(i)
    console.timeEnd(`compute_recursive_through_js(${i})`);
    console.log({
        number_of_calls: rust_boa_recursion.__wasm.get_number_of_calls(),
        max_call_depth: rust_boa_recursion.__wasm.get_max_call_depth(),
        call_stack: rust_boa_recursion.__wasm.get_call_stack(),
    });
}
