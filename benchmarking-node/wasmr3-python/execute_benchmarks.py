# -*- coding: utf-8 -*-
import os
import re
import logging
import subprocess

from config import timeout, timeout_treshold, benchmark_runs, NODE_BENCHMARK_RUNS, EXIT_STATUS_SUCCESS

def execute_benchmarks(
    setup_name: str,
    runtime_name: str,
    input_program: str,
    target_build_directory: str,
    results_file_path: str,
):
    with open(results_file_path, 'a') as results_file:
        benchmark_path = os.path.join(target_build_directory, input_program, f'{input_program}.cjs')
        times_this_combination_timed_out = 0

        for run in range(benchmark_runs):
            logging.info(f"[BENCHMARK PROGRESS {setup_name}]: PROGRAM '{input_program}' - RUN [{run+1}/{benchmark_runs}]")

            if times_this_combination_timed_out >= timeout_treshold:
                logging.warning(f"[BENCHMARK PROGRESS {setup_name}]: PROGRAM '{input_program}' - at run {run+1} I decide to quit (timed out more than {timeout_treshold} times!)]")
                return

            try:
                # run benchmark & write to file
                bench_run_result = subprocess.run(
                    ['bash', '-c', f'node --experimental-wasm-multi-memory {benchmark_path}'],
                    capture_output=True,
                    text=True,
                    timeout=timeout,
                )

            except subprocess.TimeoutExpired:
                logging.warning(f'[setup:{setup_name},benchmark:{input_program},runtime:{runtime_name}] timeout - {timeout}')
                results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","0", "timeout {timeout}", "s"\n')
                results_file.flush()
                times_this_combination_timed_out += 1
                continue

            if bench_run_result.returncode is not EXIT_STATUS_SUCCESS:
                logging.warning(f'[setup:{setup_name},benchmark:{input_program},runtime:{runtime_name}] error!')
                results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","0", "error", "s"\n')
                results_file.flush()
                return

            # At this point the run was a success, assert stdout reports run result

            allowed_ignore_pattern = r'Wasabi: hook [\w-]+ not provided by Wasabi.analysis, I will use an empty function as a fallback'
            bench_run_result_stdout_lines = bench_run_result.stdout.strip().split('\n')

            # Now walk over subprocess' stdout, filter 'ignore pattern'
            captured_lines = []
            for bench_run_result_stdout_line in bench_run_result_stdout_lines:
                if len(bench_run_result_stdout_line) == 0: continue
                if re.match(allowed_ignore_pattern, bench_run_result_stdout_line): continue
                captured_lines += [bench_run_result_stdout_line]

            # assert exactly 'benchmark_runs' amount of lines are kept as 'relevant' here!
            assert len(captured_lines) == NODE_BENCHMARK_RUNS, f'captured_lines:{captured_lines}\nNODE_BENCHMARK_RUNS:{NODE_BENCHMARK_RUNS}'
            total_time = 0
            time_unit = 'ms'
            for benchmark_report_line in captured_lines:
                benchmark_report_line = benchmark_report_line.strip()
                #                           input_program      run            performance
                #                             <------>        <--->     <-------------------->
                performance_regex_pattern = r'([\w-]+)\ \(run (\d+)\): ((?:\d)+(?:\.(?:\d)+)?)'
                pattern_match = re.match(performance_regex_pattern, benchmark_report_line)
                assert pattern_match is not None
                [re_input_program, re_run, re_performance] = [pattern_match.group(i) for i in [1, 2, 3]]
                assert input_program == re_input_program
                assert benchmark_report_line == f'{re_input_program} (run {re_run}): {re_performance}'

                total_time += float(re_performance)

                #                     'setup ✅,      runtime ✅,      input_program ✅, run-iter ✅, performance ✅,   time-unit ✅\n'
                results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","{re_run}","{re_performance}","{time_unit}"\n')
                results_file.flush()

            logging.info(f' -> {NODE_BENCHMARK_RUNS} took in total {total_time}{time_unit}')
