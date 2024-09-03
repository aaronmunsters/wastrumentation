const rust_recursion = require("./rust_boa_recursion.js");

const REPETITIONS = 1;

// Target: input to `compute_recursive` is 'slow, large'
const i = 13;

console.time(`[UNINSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
for (let r = 0; r <= REPETITIONS; r++) rust_recursion.compute_recursive(i);
console.timeEnd(`[UNINSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
