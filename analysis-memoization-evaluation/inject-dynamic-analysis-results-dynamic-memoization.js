const [_node, _script, dynamic_analysis_results_path, target_rust_code_path, output_functions_indices_path] = process.argv;
const fs = require("fs");

const dynamic_analysis_results_string = fs.readFileSync(dynamic_analysis_results_path, "utf8");
const dynamic_analysis_results = JSON.parse(dynamic_analysis_results_string);
const target_functions = [];

const THRESHOLD = 754000;

for (function_index in dynamic_analysis_results) {
    const runtime_call_count = dynamic_analysis_results[function_index];
    if (runtime_call_count >= THRESHOLD) {
        target_functions.push(function_index);
        console.log(`(INCLUDED):: ${function_index} =CALL=COUNT=> ${runtime_call_count}`);
    } else {
        console.log(`(NOT INCL):: ${function_index} =CALL=COUNT=> ${runtime_call_count}`);
    }
}

fs.writeFileSync(
    output_functions_indices_path,
    target_functions.join(" "),
    { flag: "w+", encoding: "utf8", flush: true },
);
