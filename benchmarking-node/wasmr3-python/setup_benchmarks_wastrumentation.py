# -*- coding: utf-8 -*-
import os
import re
import shutil
import logging
import subprocess

from config import bench_suite_benchmarks_path_wastrumentation

def setup_benchmarks_wastrumentation(
    node_wasm_wrap_path: str,
    benchmark: str,
    benchmark_path: str,
    analysis_name: str,
    analysis_path: str,
    analysis_hooks: list[str],
    intra_vm_runs: int,
):
    os.makedirs(bench_suite_benchmarks_path_wastrumentation, exist_ok=True)

    # Output path of [benchmark directory / program]
    benchmark_directory_wastrumentation_instrumented = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name, benchmark)
    benchmark_path_wastrumentation_instrumented = os.path.join(benchmark_directory_wastrumentation_instrumented, f'{benchmark}.wasm')
    os.makedirs(benchmark_directory_wastrumentation_instrumented, exist_ok=True)

    # Setup default wrapper [uninstrumented]
    wrapper_output_path = os.path.join(benchmark_directory_wastrumentation_instrumented, f'{benchmark}.cjs')
    shutil.copy(node_wasm_wrap_path, wrapper_output_path)

    # Replace the template with actual values
    wrapper_content = open(wrapper_output_path, 'r').read()
    for pattern, replacement in [
        [r'INPUT_PROGRAM_PATH', f'{benchmark_path_wastrumentation_instrumented}'],
        [r'INPUT_NAME', f'{benchmark}'],
        [r'NODE_BENCHMARK_RUNS', f'{intra_vm_runs}'],
    ]: wrapper_content = re.sub(pattern, replacement, wrapper_content)
    # write to template
    open(wrapper_output_path, 'w').write(wrapper_content)

    if os.path.exists(benchmark_path_wastrumentation_instrumented):
        logging.info(f'[WASTRUMENTATION INSTRUMENTATION PHASE] instrumented already exists; skipping: {analysis_name}/{benchmark}.wasm')
        return

    # copy over input.wasm
    shutil.copy(benchmark_path, benchmark_path_wastrumentation_instrumented)

    # Setup wastrumentation instrumentation infrastructure
    hooks = ' '.join(analysis_hooks)

    subprocess.run([
        'bash', '-c', f"""                                                  \
        cargo run --bin wastrumentation-cli --                              \
            --input-program-path "{benchmark_path}"                         \
            --rust-analysis-toml-path "{analysis_path}/Cargo.toml"          \
            --hooks {hooks}                                                 \
            --output-path "{benchmark_path_wastrumentation_instrumented}"
        """
    ])
