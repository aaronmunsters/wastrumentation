/*
THIS IS A TEMPLATE; THE FOLLOWING TEMPS WILL BE REPLACED BY A PRE-PROCESSOR:

- input program       = INPUT_NAME
- node benchmark runs = NODE_BENCHMARK_RUNS
*/

const readFileSync = require("fs").readFileSync;
const input_program = readFileSync("./INPUT_NAME.wasm");

(async () => {
    for (let i = 1; i <= NODE_BENCHMARK_RUNS; i++) {
        const instantiated_source = await WebAssembly.instantiate(input_program, {});
        const { _start } = instantiated_source.instance.exports;
        console.time(`INPUT_NAME (run ${i})`);
        _start()
        console.timeEnd(`INPUT_NAME (run ${i})`);
    }
})()
