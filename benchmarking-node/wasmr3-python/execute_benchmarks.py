# -*- coding: utf-8 -*-
import os
import csv
import re
import logging
import subprocess

from allowed_failures import identify_error
from config import timeout, timeout_treshold, benchmark_runs, NODE_BENCHMARK_RUNS, EXIT_STATUS_SUCCESS

def execute_benchmarks(
    runtime: str,
    platform: str,
    analysis: str | None,
    input_program: str,
    csv_writer: csv.DictWriter,
    target_build_directory: str,
):
    benchmark_path = os.path.join(target_build_directory, input_program, f'{input_program}.cjs')
    times_this_combination_timed_out = 0

    for run in range(benchmark_runs):
        default_csv_report = {
            'runtime': runtime,
            'platform': platform,
            'analysis': analysis,
            'input_program': input_program,
        }

        logging.info(f"[BENCHMARK PROGRESS {platform}-{analysis}]: PROGRAM '{input_program}' - RUN [{run+1}/{benchmark_runs}]")

        if times_this_combination_timed_out >= timeout_treshold:
            logging.warning(f"[BENCHMARK PROGRESS {platform}-{analysis}]: PROGRAM '{input_program}' - at run {run+1} I decide to quit (timed out more than {timeout_treshold} times!)]")
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
            logging.warning(f'[platform:{platform},analysis:{analysis},benchmark:{input_program},runtime:{runtime}] timeout - {timeout}')
            csv_values = default_csv_report.copy()
            csv_values['memory_usage'] = None
            csv_values['completion_time'] = None
            csv_values['time_unit'] = None
            csv_values['exception'] = False
            csv_values['exception_reason'] = None
            csv_values['timeout'] = True
            csv_values['timeout_amount'] = timeout
            csv_values['runtime_iteration'] = 0,

            csv_writer.writerow(csv_values)
            times_this_combination_timed_out += 1
            continue

        if bench_run_result.returncode is not EXIT_STATUS_SUCCESS:
            identified_error = identify_error(bench_run_result.stderr)
            logging.warning(f'[platform:{platform},analysis:{analysis},benchmark:{input_program},runtime:{runtime}] error!')
            csv_values = default_csv_report.copy()
            csv_values['memory_usage'] = None
            csv_values['completion_time'] = None
            csv_values['time_unit'] = None
            csv_values['exception'] = True
            csv_values['exception_reason'] = identified_error
            csv_values['timeout'] = False
            csv_values['timeout_amount'] = timeout
            csv_values['runtime_iteration'] = 0

            csv_writer.writerow(csv_values)
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

        # assert `benchmark_runs * 2` lines are kept as relevant here; `benchmark_runs` reporting performance and memory
        assert len(captured_lines) == NODE_BENCHMARK_RUNS * 2, f'captured_lines:{captured_lines}\nNODE_BENCHMARK_RUNS:{NODE_BENCHMARK_RUNS}'
        execution_bench: list[tuple[float, int]] = []

        total_time = 0
        time_unit = 'ms'
        for benchmark_report_line in captured_lines:
            benchmark_report_line = benchmark_report_line.strip()
            #                           input_program      run            performance
            #                             <------>        <--->     <-------------------->
            pattern_performance_regex = r'([\w-]+)\ \(run (\d+)\): ((?:\d)+(?:\.(?:\d)+)?)'
            #                                               bytes
            #                                               <--->
            pattern_memory_regex = r'([\w-]+) memory usage in bytes: (\d+)'

            pattern_match_performance = re.match(pattern_performance_regex, benchmark_report_line)
            pattern_match_memory = re.match(pattern_memory_regex, benchmark_report_line)


            # assert line matches at least performance or memory report
            assert pattern_match_performance is not None or pattern_match_memory is not None

            execution_time = None
            if pattern_match_performance is not None:
                [re_input_program, re_run, re_performance] = [pattern_match_performance.group(i) for i in [1, 2, 3]]
                assert input_program == re_input_program
                assert benchmark_report_line == f'{re_input_program} (run {re_run}): {re_performance}'
                execution_time = float(re_performance)
                total_time += execution_time
                logging.info(f' -> single run took {execution_time}{time_unit}')

            memory_usage_bytes = None
            if pattern_match_memory is not None:
                [re_input_program, re_bytes] = [pattern_match_memory.group(i) for i in [1, 2]]
                assert input_program == re_input_program
                assert benchmark_report_line == f'{re_input_program} memory usage in bytes: {re_bytes}'
                memory_usage_bytes = int(re_bytes)
                logging.info(f' -> single run took {memory_usage_bytes} bytes of wokring memory')

            if execution_time is None: continue
            if memory_usage_bytes is None: continue
            new_execution_bench: tuple[float, int] = (execution_time, memory_usage_bytes)
            execution_bench += [new_execution_bench]

        logging.info(f' -> {NODE_BENCHMARK_RUNS} took in total {total_time}{time_unit}')
        for iteration, (execution_time, memory_usage) in enumerate(execution_bench):
            csv_values = default_csv_report.copy()
            csv_values['memory_usage'] = memory_usage
            csv_values['completion_time'] = execution_time
            csv_values['time_unit'] = time_unit
            csv_values['exception'] = False
            csv_values['exception_reason'] = None
            csv_values['timeout'] = False
            csv_values['timeout_amount'] = timeout
            csv_values['runtime_iteration'] = iteration
            csv_writer.writerow(csv_values)
