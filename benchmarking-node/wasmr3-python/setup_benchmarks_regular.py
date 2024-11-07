# -*- coding: utf-8 -*-
import os
import re
import logging

from config import bench_suite_benchmarks_path

def setup_benchmarks_regular(
        node_wasm_wrap_path: str,
        benchmarks: dict[str, str],
        intra_vm_runs: int,
):
    if len(benchmarks) == 0: return

    # read wrapper_template to variable
    wrapper_template_fd = open(node_wasm_wrap_path, 'r')
    wrapper_template = wrapper_template_fd.read()
    wrapper_template_fd.close()

    # assert template layout
    for (benchmark, benchmark_path) in benchmarks.items():
        # Setup default wrapper [uninstrumented]
        wrapper_path = os.path.join(bench_suite_benchmarks_path, benchmark, f'{benchmark}.cjs')

        # Generate wrapper_content from template
        wrapper_content = wrapper_template
        for pattern, replacement in [
            [r'INPUT_PROGRAM_PATH', f'{benchmark_path}'],
            [r'INPUT_NAME', f'{benchmark}'],
            [r'NODE_BENCHMARK_RUNS', f'{intra_vm_runs}'],
        ]:
            if re.search(pattern, wrapper_content) is None:
                logging.warning(f"Rewritting '{pattern}' to '{replacement}' will not happen since '{pattern}' did not occur in {wrapper_content}")
            wrapper_content = re.sub(pattern, replacement, wrapper_content)

        # Write wrapper_content to wrapper_path
        with open(wrapper_path, 'w') as wrapper_path_fd:
            wrapper_path_fd.write(wrapper_content)
