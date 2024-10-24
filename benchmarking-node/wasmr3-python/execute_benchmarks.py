# -*- coding: utf-8 -*-
import os
import re
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
    os.chdir(bench_suite_benchmarks_path)
    results_file = open(results_file_path, 'a')

    # Run benchmarks

    timeout_seconds = f'{timeout}s'
    print(f'Starting benchmarks for {len(input_programs)} input programs, {benchmark_runs} running on {runtime_name}...')
    for count, input_program in enumerate(input_programs):
        benchmark_directory_path = os.path.join(target_build_directory, input_program)
        os.chdir(benchmark_directory_path)
        for run in range(benchmark_runs):
            print(
                f"[BENCHMARK PROGRESS {setup_name}]: PROGRAM [{count+1}/{len(input_programs)}] '{input_program}' - RUN [{run+1}/{benchmark_runs}]",
                end='',
            )

            # run benchmark & write to file
            bench_run_result = subprocess.run(
                [
                    'bash', '-c', f"""                    \
                    timeout {timeout_seconds}             \
                    node --experimental-wasm-multi-memory \
                        {benchmark_directory_path}/{input_program}.cjs
                    """
                ],
                capture_output=True,
                text=True
            )

            if bench_run_result.returncode == EXIT_STATUS_TIMEOUT:
                print(f' ... timeout! Took >= {timeout_seconds} seconds!')
                results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","0", "timeout {timeout_seconds}", "s"\n')
                results_file.flush()
                continue

            # At this point the run was a success, assert stdout reports run result

            # Blacklist lines that are of the nature 'Wasabi: hook <...> not provided by Wasabi.analysis, I will use an empty function as a fallback'
            allowed_ignore_pattern = r'Wasabi: hook [\w-]+ not provided by Wasabi.analysis, I will use an empty function as a fallback'
            bench_run_result_stdout_lines = bench_run_result.stdout.strip().split('\n')

            # Now walk over subprocess' stdout, filter 'ignore pattern'
            captured_lines = []
            for bench_run_result_stdout_line in bench_run_result_stdout_lines:
                if len(bench_run_result_stdout_line) == 0:
                    continue
                if re.match(allowed_ignore_pattern, bench_run_result_stdout_line):
                    continue
                captured_lines += [bench_run_result_stdout_line]

            # assert exactly 'benchmark_runs' amount of lines are kept as 'relevant' here!
            assert len(captured_lines) == benchmark_runs
            total_time = 0
            time_unit = ''
            for benchmark_report_line in captured_lines:
                benchmark_report_line = benchmark_report_line.strip()
                #                           input_program      run             performance   time-unit
                #                             <------>        <--->     <---------------------><--->
                performance_regex_pattern = r'([\w-]+)\ \(run (\d+)\): ((?:\d)+(?:\.(?:\d)+)?)(\w+)'
                pattern_match = re.match(performance_regex_pattern, benchmark_report_line)
                assert pattern_match is not None
                [re_input_program, re_run, re_performance, re_time_unit] = [pattern_match.group(i) for i in [1, 2, 3, 4]]
                assert input_program == re_input_program
                assert benchmark_report_line == f'{re_input_program} (run {re_run}): {re_performance}{re_time_unit}'

                total_time += float(re_performance)
                time_unit = re_time_unit

                #                     'setup ✅,      runtime ✅,      input_program ✅, run-iter ✅, performance ✅,   time-unit ✅\n'
                results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","{re_run}","{re_performance}","{re_time_unit}"\n')
                results_file.flush()

            print(f' -> took {total_time}{time_unit}')
