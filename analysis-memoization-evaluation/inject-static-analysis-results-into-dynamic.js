const [_node, _script, static_analysis_results_path, target_rust_code_path] = process.argv;
const fs = require("fs");

const static_analysis_results_string = fs.readFileSync(static_analysis_results_path, "utf8");
const static_analysis_results = JSON.parse(static_analysis_results_string);
const map_size = static_analysis_results.length;

let target_rust_code = fs.readFileSync(target_rust_code_path, "utf8");

//
//                  <-----------G1-----------><---><--------------G2--------------->
const map_target = /(const MAP_SIZE: usize = )(\d+)(; \/\/ <TO_CODE_GEN {MAP_SIZE}>)/g;
console.assert(
    target_rust_code.matchAll(map_target).next().value !== undefined,
    `Could not find gen location to increment: ${map_target}`
);
target_rust_code = target_rust_code.replace(map_target, `$1${map_size}$3`)

const map_increment_code_target = /((\s)*)\/\/ <TO_CODE_GEN {MAP_INCREMENT}>/g
let map_increment_code_match = target_rust_code.matchAll(map_increment_code_target).next();
console.assert(
    map_increment_code_match.value !== undefined,
    `Could not find gen location to increment: ${map_increment_code_target}`
);
let { value } = map_increment_code_match;
const [_match, space_group] = value;

const map_increment_code = static_analysis_results.map((function_index, map_index, _array) =>
    `${space_group}${function_index} => map[${map_index}] = map[${map_index}] + 1,`
).join("\n");

target_rust_code = target_rust_code.replace(map_increment_code_target, map_increment_code);
console.log(target_rust_code)
