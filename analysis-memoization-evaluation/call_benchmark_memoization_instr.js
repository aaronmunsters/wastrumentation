const [_node, _script, bound_library, instrumented_functions_count_arg, instrumentation_output_path] = process.argv;
console.log(`const rust_recursion = require(${bound_library});`);
const rust_recursion = require(bound_library);

const instrumented_functions_count = parseInt(instrumented_functions_count_arg);
const REPETITIONS = 1;

const pure_functions_that_are_called = new Map();

// Target: input to `compute_recursive` is 'slow, large'
const i = 13;

console.log(`CACHE_SIZE_REPORT == ${rust_recursion.__wasm.CACHE_SIZE_REPORT()}`);
console.time(`[INSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
for (let r = 0; r <= REPETITIONS; r++) rust_recursion.compute_recursive(i);
console.timeEnd(`[INSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
console.log(`CACHE_SIZE_REPORT == ${rust_recursion.__wasm.CACHE_SIZE_REPORT()}`);
console.log(`CACHE_HIT_REPORT == ${rust_recursion.__wasm.CACHE_HIT_REPORT()}`);
