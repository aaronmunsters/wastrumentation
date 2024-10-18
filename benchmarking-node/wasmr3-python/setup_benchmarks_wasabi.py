# -*- coding: utf-8 -*-
import os
import re
import shutil
import subprocess

from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wasabi
from config import NODE_BENCHMARK_RUNS

def setup_benchmarks_wasabi(
    node_wasm_wrap_path: str,
    input_programs: list[str],
    analysis_path: str,
):
    if not os.path.exists(bench_suite_benchmarks_path_wasabi):
        os.makedirs(bench_suite_benchmarks_path_wasabi, exist_ok=True)

    for input_program in input_programs:
        benchmark_directory = os.path.join(bench_suite_benchmarks_path, input_program)
        benchmark_path = os.path.join(benchmark_directory, f'{input_program}.wasm')
        benchmark_directory_wasabi_instrumented = os.path.join(bench_suite_benchmarks_path_wasabi, input_program)
        benchmark_path_wasabi_instrumented = os.path.join(benchmark_directory_wasabi_instrumented, f'{input_program}.wasm')
        if not os.path.exists(benchmark_directory_wasabi_instrumented):
            os.makedirs(benchmark_directory_wasabi_instrumented, exist_ok=True)

        # copy over input.wasm
        shutil.copy(benchmark_path, benchmark_path_wasabi_instrumented)

        # Setup wasabi instrumentation infrastructure

        # The following command:
        #
        #   wasabi --node --output-dir [dir] [<input>.wasm]
        #
        # will output a `<input>.wasabi.js` file and
        # an instrumented `<input>.wasm` file in the
        # output directory [dir]
        hooks = [
                'nop',
                'unreachable',
                'if',
                'br',
                'br_if',
                'br_table',
                'drop',
                'select',
                'memory_size',
                'memory_grow',
                'unary',
                'binary',
                'load',
                'store',
                'local',
                'global',
                'call',
                'const',
                'begin',
                'return',
        ]
        hooks = ' '.join(map(lambda hook: f'--hooks {hook}', hooks))

        subprocess.run([
            'bash', '-c', f"""                                  \
            wasabi                                              \
                --node                                          \
                --output-dir                                    \
                    "{benchmark_directory_wasabi_instrumented}" \
                {hooks}                                         \
                "{benchmark_path_wasabi_instrumented}"
            """
        ])

        wasabi_generated_script_path_old = os.path.join(benchmark_directory_wasabi_instrumented, f'{input_program}.wasabi.js')
        wasabi_generated_script_path = os.path.join(benchmark_directory_wasabi_instrumented, f'{input_program}.wasabi.cjs')
        shutil.move(wasabi_generated_script_path_old, wasabi_generated_script_path)

        # Setup default wrapper [uninstrumented]
        wrapper_output_path = os.path.join(benchmark_directory_wasabi_instrumented, f'{input_program}.cjs')
        shutil.copy(node_wasm_wrap_path, wrapper_output_path)

        patch_js_to_cjs(benchmark_directory_wasabi_instrumented, wasabi_generated_script_path)

        # Replace the template with actual values
        wrapper_content_template_filled = open(wrapper_output_path, 'r').read()
        for pattern, replacement in [
            [r'INPUT_NAME', f'{input_program}'],
            [r'NODE_BENCHMARK_RUNS', f'{NODE_BENCHMARK_RUNS}'],
        ]: wrapper_content_template_filled = re.sub(pattern, replacement, wrapper_content_template_filled)

        # Inject call to wasabi
        wrapper_content = ''
        wrapper_content += f'globalThis.Wasabi = require("./{input_program}.wasabi.cjs");\n'
        wrapper_content += f'const user_hooks = require("{analysis_path}");\n'
        wrapper_content += wrapper_content_template_filled

        # write to template
        open(wrapper_output_path, 'w').write(wrapper_content)


        wrapper_content = open(wrapper_output_path, 'r').read()
        for pattern, replacement in [
            [r'INPUT_NAME', f'{input_program}'],
            [r'NODE_BENCHMARK_RUNS', f'{NODE_BENCHMARK_RUNS}'],
        ]: wrapper_content = re.sub(pattern, replacement, wrapper_content)
        # write to template
        open(wrapper_output_path, 'w').write(wrapper_content)

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

    # Replace in {input_program}.wasabi.cjs
    wasabi_generated_script_content = open(wasabi_generated_script_path, 'r').read()
    wasabi_generated_script_content = re.sub(js_pattern, cjs_replacement, wasabi_generated_script_content)
    open(wasabi_generated_script_path, 'w').write(wasabi_generated_script_content)
