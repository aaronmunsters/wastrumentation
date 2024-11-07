# -*- coding: utf-8 -*-
import os

# Benchmark constants
benchmark_runs = 2
NODE_BENCHMARK_RUNS = 2

# Directory paths
analyses_directory: str = os.path.abspath('analyses')
working_directory: str = os.path.abspath('working-dir')

# NodeJS analysis wrapper
node_wasm_wrap_path: str = os.path.abspath('node-wasm-wrap.cjs')

# Output files
path_results_file_regular = os.path.join(working_directory, 'runtime-analysis-regular.csv')
path_results_file_wasabi = os.path.join(working_directory, 'runtime-analysis-wasabi.csv')
path_results_file_wastrumentation = os.path.join(working_directory, 'runtime-analysis-wastrumentation.csv')

# Output files
path_code_size_results_file_regular = os.path.join(working_directory, 'code-size-analysis-regular.csv')
path_code_size_results_file_wasabi = os.path.join(working_directory, 'code-size-analysis-wasabi.csv')
path_code_size_results_file_wastrumentation = os.path.join(working_directory, 'code-size-analysis-wastrumentation.csv')

# Output files
path_executes_once_results_file_regular = os.path.join(working_directory, 'executes-once-analysis-regular.csv')
path_executes_once_results_file_wasabi = os.path.join(working_directory, 'executes-once-analysis-wasabi.csv')
path_executes_once_results_file_wastrumentation = os.path.join(working_directory, 'executes-once-analysis-wastrumentation.csv')

# Bench suite
bench_suite_uri = 'git@github.com:sola-st/wasm-r3.git'
bench_suite_commit = '299be52000046e5d49248c4c66a21238855587d7'
bench_suite_path = os.path.join(working_directory, 'wasm-r3')
bench_suite_benchmarks_path = os.path.join(bench_suite_path, 'benchmarks')
bench_suite_benchmarks_path_wasabi = os.path.join(bench_suite_path, 'benchmarks_wasabi')
bench_suite_benchmarks_path_wastrumentation = os.path.join(bench_suite_path, 'benchmarks_wastrumentation')

# Timeout and exit status
timeout = 300 # seconds
EXIT_STATUS_SUCCESS = 0
minimum_major_node_version = 22
minimum_major_wasm_merge_version = 119
