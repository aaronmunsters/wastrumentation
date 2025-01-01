/*
THIS IS A TEMPLATE; THE FOLLOWING TEMPS WILL BE REPLACED BY A PRE-PROCESSOR:

- input program       = INPUT_NAME
- node benchmark runs = NODE_BENCHMARK_RUNS
*/

const { performance, PerformanceObserver } = require("node:perf_hooks");
const readFileSync = require("fs").readFileSync;
const input_program = readFileSync("INPUT_PROGRAM_PATH");
const process = require('process')

const observer = new PerformanceObserver((performance_observer_entry_list) => {
    for (const performance_entry of performance_observer_entry_list.getEntries()) {
        let report_string = `${performance_entry.name}: ${performance_entry.duration}`;
        console.log(report_string);
    }
});

observer.observe({type: 'measure'});

let reported = false;
function report_memory_once() {
    // Guard that report happens only once!
    if (reported) return;
    const current_memory_usage_dictionary = process.memoryUsage();
    const {rss, heapTotal, heapUsed, external, arrayBuffers} = current_memory_usage_dictionary;
    const current_memory_usage = rss + heapTotal + heapUsed + external + arrayBuffers;
    console.log(`INPUT_NAME memory usage in bytes: ${current_memory_usage}`);
    reported = true;
}

(async () => {
    for (let i = 1; i <= NODE_BENCHMARK_RUNS; i++) {
        // Instantiate & retrieve export
        const instantiated_source = await WebAssembly.instantiate(input_program, {});
        const { _start } = instantiated_source.instance.exports;

        // Benchmark
        const mark_name = `INPUT_NAME (run ${i})`;
        performance.mark(mark_name);
        _start()
        performance.measure(mark_name, mark_name);

        // Report memory usage once
        report_memory_once();
    }
})()
