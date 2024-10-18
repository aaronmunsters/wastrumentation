# -*- coding: utf-8 -*-
import os
import subprocess

from config import timeout, benchmark_runs, EXIT_STATUS_TIMEOUT
from config import bench_suite_benchmarks_path

def execute_benchmarks(
    setup_name: str,
    runtime_name: str,
    input_programs: list[str],
    target_build_directory: str,
    results_file_path: str,
):
    # cd to polybench_directory
    os.chdir(bench_suite_benchmarks_path)

    # Run benchmarks
    results_file = open(results_file_path, 'w')
    results_file.write('setup,runtime,input-program,performance\n')
    results_file.flush()

    # Trap exit signals (Ctrl+C)
    timeout_seconds = f'{timeout}s'

    print(f'Starting benchmarks for {len(input_programs)} input programs, {benchmark_runs} running on {runtime_name}...')
    for count, input_program in enumerate(input_programs):
        benchmark_directory_path = os.path.join(target_build_directory, input_program)
        os.chdir(benchmark_directory_path)
        for run in range(benchmark_runs):
            print(f"[BENCHMARK PROGRESS {setup_name}]: PROGRAM '{input_program}' [{count}/{len(input_programs)}] - RUN [{run}/{benchmark_runs}]")

            results_file.write(f'"{setup_name}","{runtime_name}","{input_program}",')
            results_file.flush()

            # run benchmark & write to file
            result = subprocess.run([
                'bash', '-c', f"""                    \
                timeout {timeout_seconds}             \
                node --experimental-wasm-multi-memory \
                    {benchmark_directory_path}/{input_program}.cjs
                """
            ])

            if result.returncode == EXIT_STATUS_TIMEOUT:
                results_file.write(f'timeout {timeout_seconds}\n')
                results_file.flush()
