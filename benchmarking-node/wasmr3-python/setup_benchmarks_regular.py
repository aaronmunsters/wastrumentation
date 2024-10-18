# -*- coding: utf-8 -*-
import os
import re
import shutil

from config import bench_suite_benchmarks_path
from config import NODE_BENCHMARK_RUNS

def setup_benchmarks_regular(
    node_wasm_wrap_path: str,
    input_programs: list[str]
):
    os.chdir(bench_suite_benchmarks_path)
    for input_program in input_programs:
        # Setup default wrapper [uninstrumented]
        wrapper_output_path = os.path.join(bench_suite_benchmarks_path, input_program, f'{input_program}.cjs')
        shutil.copy(node_wasm_wrap_path, wrapper_output_path)

        # Replace the template with actual values
        wrapper_content = open(wrapper_output_path, 'r').read()
        for pattern, replacement in [
            [r'INPUT_NAME', f'{input_program}'],
            [r'NODE_BENCHMARK_RUNS', f'{NODE_BENCHMARK_RUNS}'],
        ]: wrapper_content = re.sub(pattern, replacement, wrapper_content)
        # write to template
        open(wrapper_output_path, 'w').write(wrapper_content)
