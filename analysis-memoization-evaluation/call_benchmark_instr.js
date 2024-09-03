const fs = require("fs");

const [_node, _script, bound_library, instrumented_functions_count_arg, pure_function_indices_path, instrumentation_output_path] = process.argv;
console.log(`const rust_recursion = require(${bound_library});`);
const rust_recursion = require(bound_library);


const instrumented_functions_count = parseInt(instrumented_functions_count_arg);
const REPETITIONS = 1;
const function_indices = JSON.parse(fs.readFileSync(pure_function_indices_path, { encoding: "utf8" }));

const pure_functions_that_are_called = new Map();

// Target: input to `compute_recursive` is 'slow, large'
const i = 13;

console.time(`[INSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
for (let r = 0; r <= REPETITIONS; r++) rust_recursion.compute_recursive(i);
console.timeEnd(`[INSTRUMENTED] ${REPETITIONS} x compute_recursive(${i})`);
for (let counter_index = 0; counter_index < instrumented_functions_count; counter_index++) {
    const function_index_count = rust_recursion.__wasm.get_calls_for(counter_index);
    if (function_index_count > 0)
        pure_functions_that_are_called.set(function_indices[counter_index], function_index_count);
}

const analysis_result = JSON.stringify(Object.fromEntries(pure_functions_that_are_called));
fs.writeFileSync(instrumentation_output_path, analysis_result, { flag: "w+", encoding: "utf8", flush: true });
