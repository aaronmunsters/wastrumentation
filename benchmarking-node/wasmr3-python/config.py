# -*- coding: utf-8 -*-
import os

# Directory paths
analyses_directory: str = os.path.abspath('analyses')
working_directory: str = os.path.abspath('working-dir')

# NodeJS analysis wrapper
node_wasm_wrap_path: str = os.path.abspath('node-wasm-wrap.cjs')

# Output files
path_code_size = os.path.join(working_directory, 'code-sizes.csv')
path_executes_once = os.path.join(working_directory, 'executes-once.csv')
path_execution_bench = os.path.join(working_directory, 'executes-bench.csv')

# Bench suite
bench_suite_uri = 'https://github.com/sola-st/wasm-r3.git'
bench_suite_commit = '299be52000046e5d49248c4c66a21238855587d7'
bench_suite_path = os.path.join(working_directory, 'wasm-r3')
bench_suite_benchmarks_path = os.path.join(bench_suite_path, 'benchmarks')
bench_suite_benchmarks_path_wasabi = os.path.join(bench_suite_path, 'benchmarks_wasabi')
bench_suite_benchmarks_path_wastrumentation = os.path.join(bench_suite_path, 'benchmarks_wastrumentation')

# Timeout and exit status
timeout = 300 # seconds
timeout_treshold = 3 # after timing out `timeout_treshold` times, quit this benchmark!
EXIT_STATUS_SUCCESS = 0
minimum_major_node_version = 22
minimum_major_wasm_merge_version = 119

KEY_runtime = 'runtime'
KEY_platform = 'platform'
KEY_analysis = 'analysis'
KEY_input_program = 'input_program'
KEY_memory_usage = 'memory_usage'
KEY_completion_time = 'completion_time'
KEY_time_unit = 'time_unit'
KEY_exception = 'exception'
KEY_exception_reason = 'exception_reason'
KEY_timeout = 'timeout'
KEY_timeout_amount = 'timeout_amount'

executes_once_field_names = [
    KEY_runtime,
    KEY_platform,
    KEY_analysis,
    KEY_input_program,
    KEY_memory_usage,
    KEY_completion_time,
    KEY_time_unit,
    KEY_exception,
    KEY_exception_reason,
    KEY_timeout,
    KEY_timeout_amount,
]

KEY_runtime_iteration = 'runtime_iteration' # this is the only addition for 'execution_bench_field_names'

execution_bench_field_names = [
    KEY_runtime,
    KEY_platform,
    KEY_analysis,
    KEY_input_program,
    KEY_memory_usage,
    KEY_completion_time,
    KEY_runtime_iteration,
    KEY_time_unit,
    KEY_exception,
    KEY_exception_reason,
    KEY_timeout,
    KEY_timeout_amount,
]

KEY_size_bytes = 'size_bytes'

code_size_field_names = [
    KEY_platform,
    KEY_analysis,
    KEY_input_program,
    KEY_size_bytes,
]
