# -*- coding: utf-8 -*-
import shutil
import os
import subprocess

from fetch_browser import FireFoxFetcher
from config import timeout, benchmark_runs, EXIT_STATUS_TIMEOUT
from config import polybench_directory, results_file_regular_name, working_directory
from benchmark_datasets import BenchmarkDatasetSizes

def execute_benchmarks(
    setup_name: str,
    browser_binary_path: str,
    browser_version: str,
    target_build_directory: str,
    results_file_path: str,
    benchmark_dataset_sizes: BenchmarkDatasetSizes,
):
    # cd to polybench_directory
    os.chdir(polybench_directory)

    # Run benchmarks
    shutil.rmtree('firefox-profile', ignore_errors=True)
    os.makedirs('firefox-profile', exist_ok=True)
    firefox_args = f"--headless -no-remote -profile {os.path.abspath('firefox-profile')}"

    results_file = open(results_file_path, 'w')
    results_file.write('browser,runtime_environment,benchmark,performance\n')
    results_file.flush()

    # Trap exit signals (Ctrl+C)

    timeout_seconds = f'{timeout}s'
    benchmarks = benchmark_dataset_sizes.benchmark_dataset_sizes.keys()

    print(f'Starting benchmarks for {len(benchmarks)} programs, {benchmark_runs} runs on {browser_version}...')
    for count, benchmark in enumerate(benchmarks):
        for run in range(benchmark_runs):
            print(f"[BENCHMARK PROGRESS {setup_name}]: PROGRAM '{benchmark}' [{count}/{len(benchmarks)}] - RUN [{run}/{benchmark_runs}]")

            results_file.write(f'{browser_version},"{setup_name}", {benchmark}, ')
            results_file.flush()

            target_benchmark_html_path = os.path.join(target_build_directory, f'{benchmark}.html')

            # run benchmark & write to file
            result = subprocess.run([
                'bash', '-c', f"""                                                              \
                timeout {timeout_seconds}                                                       \
                emrun                                                                           \
                    --log_stdout   "{results_file_path}"   `# Write findings to file          ` \
                    --browser      "{browser_binary_path}" `# Rely on downloaded browser      ` \
                    --browser_args "{firefox_args}"        `# Pass custom arguments to browser` \
                    --kill_exit                            `# Kill browser process on exit    ` \
                    {target_benchmark_html_path}           `# Target benchmark                `
                """
            ])

            if result.returncode == EXIT_STATUS_TIMEOUT:
                results_file.write(f'timeout {timeout_seconds}\n')
                results_file.flush()
