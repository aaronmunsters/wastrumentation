# -*- coding: utf-8 -*-
import os
import re
import shutil
import subprocess

from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wastrumentation
from config import NODE_BENCHMARK_RUNS

def setup_benchmarks_wastrumentation(
    node_wasm_wrap_path: str,
    input_programs: list[str],
    analysis_path: str,
):
    if not os.path.exists(bench_suite_benchmarks_path_wastrumentation):
        os.makedirs(bench_suite_benchmarks_path_wastrumentation, exist_ok=True)

    for input_program in input_programs:
        benchmark_directory = os.path.join(bench_suite_benchmarks_path, input_program)
        benchmark_path = os.path.join(benchmark_directory, f'{input_program}.wasm')
        benchmark_directory_wastrumentation_instrumented = os.path.join(bench_suite_benchmarks_path_wastrumentation, input_program)
        benchmark_path_wastrumentation_instrumented = os.path.join(benchmark_directory_wastrumentation_instrumented, f'{input_program}.wasm')
        if not os.path.exists(benchmark_directory_wastrumentation_instrumented):
            os.makedirs(benchmark_directory_wastrumentation_instrumented, exist_ok=True)

        # copy over input.wasm
        shutil.copy(benchmark_path, benchmark_path_wastrumentation_instrumented)

        # Setup wastrumentation instrumentation infrastructure
        hooks = [
                'call-pre',
                'call-post',
                'call-indirect-pre',
                'call-indirect-post',
                'if-then',
                'if-then-post ',
                'if-then-else',
                'if-then-else-post',
                'branch',
                'branch-if',
                'branch-table',
                'select',
                'unary',
                'binary',
                'drop',
                'return',
                'const',
                'local',
                'global',
                'store',
                'load',
                'memory-size',
                'memory-grow',
                'block-pre',
                'block-post',
                'loop-pre',
                'loop-post',
        ]
        hooks = ' '.join(hooks)

        subprocess.run([
            'bash', '-c', f"""                                                  \
            cargo run  --                                                       \
                --input-program-path "{benchmark_path}"                         \
                --rust-analysis-toml-path "{analysis_path}/Cargo.toml"          \
                --hooks {hooks}                                                 \
                --output-path "{benchmark_path_wastrumentation_instrumented}"
            """
        ])

        # Setup default wrapper [uninstrumented]
        wrapper_output_path = os.path.join(benchmark_directory_wastrumentation_instrumented, f'{input_program}.cjs')
        shutil.copy(node_wasm_wrap_path, wrapper_output_path)

        # Replace the template with actual values
        wrapper_content = open(wrapper_output_path, 'r').read()
        for pattern, replacement in [
            [r'INPUT_NAME', f'{input_program}'],
            [r'NODE_BENCHMARK_RUNS', f'{NODE_BENCHMARK_RUNS}'],
        ]: wrapper_content = re.sub(pattern, replacement, wrapper_content)
        # write to template
        open(wrapper_output_path, 'w').write(wrapper_content)
