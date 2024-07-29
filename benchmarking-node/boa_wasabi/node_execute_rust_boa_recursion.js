globalThis.Wasabi = require("./rust_boa_recursion_bg.wasabi.js");
const user_hooks = require("./log-all.js");

const rust_boa_recursion = require("./rust_boa_recursion.js");

for (let i = 1; i <= 13; i++) {
    console.time(`compute_recursive_through_js(${i})`);
    rust_boa_recursion.compute_recursive_through_js(i)
    console.timeEnd(`compute_recursive_through_js(${i})`);
    console.log(globalThis.analysis_results);
}
