# -*- coding: utf-8 -*-
import os
import re
import shutil
import subprocess
import logging

from config import bench_suite_benchmarks_path_wasabi

def setup_benchmarks_wasabi(
    node_wasm_wrap_path: str,
    benchmark: str,
    benchmark_path: str,
    analysis_name: str,
    analysis_path: str,
    analysis_hooks: list[str],
    intra_vm_runs: int,
):
    # Create bench suite dir if not exists
    os.makedirs(bench_suite_benchmarks_path_wasabi, exist_ok=True)

    # Output path of [benchmark directory / program]
    benchmark_directory_wasabi_instrumented = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name, benchmark)
    os.makedirs(benchmark_directory_wasabi_instrumented, exist_ok=True)
    benchmark_path_wasabi_instrumented = os.path.join(benchmark_directory_wasabi_instrumented, f'{benchmark}.wasm')

    # Setup default wrapper [uninstrumented]
    wrapper_output_path = os.path.join(benchmark_directory_wasabi_instrumented, f'{benchmark}.cjs')
    shutil.copy(node_wasm_wrap_path, wrapper_output_path)

    # Replace the template with actual values
    wrapper_content_template_filled = open(wrapper_output_path, 'r').read()
    for pattern, replacement in [
        [r'INPUT_PROGRAM_PATH', f'{benchmark_path_wasabi_instrumented}'],
        [r'INPUT_NAME', f'{benchmark}'],
        [r'NODE_BENCHMARK_RUNS', f'{intra_vm_runs}'],
    ]: wrapper_content_template_filled = re.sub(pattern, replacement, wrapper_content_template_filled)

    # Inject call to wasabi
    wrapper_content = ''
    wrapper_content += f'globalThis.Wasabi = require("./{benchmark}.wasabi.cjs");\n'
    wrapper_content += f'const user_hooks = require("{analysis_path}");\n'
    wrapper_content += wrapper_content_template_filled

    # write to template
    open(wrapper_output_path, 'w').write(wrapper_content)

    if os.path.exists(benchmark_path_wasabi_instrumented):
        logging.info(f'[WASABI INSTRUMENTATION PHASE] instrumented already exists; skipping: {analysis_name}/{benchmark}.wasm')
        return

    # copy over input.wasm
    shutil.copy(benchmark_path, benchmark_path_wasabi_instrumented)

    # Setup wasabi instrumentation infrastructure

    # The following shell command: `wasabi --node --output-dir [dir] [<input>.wasm]`
    # will output a `<input>.wasabi.js` file and an instrumented `<input>.wasm` file
    # in the output directory [dir]
    hooks = ' '.join(map(lambda hook: f'--hooks {hook}', analysis_hooks))

    wasabi_execute_result = subprocess.run([
        'bash', '-c', f"""                                  \
        wasabi                                              \
            --node                                          \
            --output-dir                                    \
                "{benchmark_directory_wasabi_instrumented}" \
            {hooks}                                         \
            "{benchmark_path_wasabi_instrumented}"
        """
    ])

    if wasabi_execute_result.returncode != 0:
        # TODO: this should be reported to a file somehow ...
        logging.warning(f'[WASABI INSTRUMENTATION PHASE] instrumentation for "{benchmark}" failed')

    wasabi_generated_script_path_old = os.path.join(benchmark_directory_wasabi_instrumented, f'{benchmark}.wasabi.js')
    wasabi_generated_script_path = os.path.join(benchmark_directory_wasabi_instrumented, f'{benchmark}.wasabi.cjs')
    shutil.move(wasabi_generated_script_path_old, wasabi_generated_script_path)
    patch_js_to_cjs(benchmark_directory_wasabi_instrumented, wasabi_generated_script_path)

def patch_js_to_cjs(
    benchmark_directory_wasabi_instrumented: str,
    wasabi_generated_script_path: str,
):
    js_pattern = r'\.js'
    cjs_replacement = '.cjs'
    # Replace in long.js
    long_path = os.path.join(benchmark_directory_wasabi_instrumented, 'long.js')
    long_new_path = os.path.join(benchmark_directory_wasabi_instrumented, 'long.cjs')
    long_content = open(long_path, 'r').read()
    long_content = re.sub(js_pattern, cjs_replacement, long_content)
    open(long_path, 'w').write(long_content)
    shutil.move(long_path, long_new_path)

    # Replace in {benchmark}.wasabi.cjs
    wasabi_generated_script_content = open(wasabi_generated_script_path, 'r').read()
    wasabi_generated_script_content = re.sub(js_pattern, cjs_replacement, wasabi_generated_script_content)
    open(wasabi_generated_script_path, 'w').write(wasabi_generated_script_content)
