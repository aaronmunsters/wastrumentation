# -*- coding: utf-8 -*-
import os
import re
import logging
import subprocess
import csv

from allowed_failures import identify_error
from config import timeout, EXIT_STATUS_SUCCESS
from config import KEY_timeout, KEY_exception, KEY_analysis, KEY_platform
from config import KEY_runtime, KEY_input_program, KEY_memory_usage, KEY_completion_time, KEY_time_unit, KEY_exception_reason, KEY_timeout_amount
from config import executes_once_field_names
from util import parse_boolean

def forward_success_runs_for(data_path):
    forward_success_runs = {}
    with open(data_path, 'r') as executes_once_file:
        executes_once_reader = csv.DictReader(executes_once_file, fieldnames=executes_once_field_names)
        next(executes_once_reader, None)  # skip the headers - https://stackoverflow.com/a/14257599

        for executes_once_measure in executes_once_reader:
            timed_out = parse_boolean(executes_once_measure[KEY_timeout])
            reported_errors = parse_boolean(executes_once_measure[KEY_exception])
            benchmark = executes_once_measure[KEY_input_program]
            platform = executes_once_measure[KEY_platform]
            if platform == 'uninstrumented': continue

            platform_hardcoded_key = {
                'Wastrumentation': 'wastrumentation',
                'Wasabi': 'wasabi',
            }[platform]

            if benchmark not in forward_success_runs: forward_success_runs[benchmark] = {}
            forward_success_runs[benchmark][platform_hardcoded_key] = not (timed_out or reported_errors)
    return forward_success_runs

def report_executes_once(
        runtime: str,
        platform: str,
        analysis: str | None,
        input_program: str,
        csv_writer: csv.DictWriter,
        target_build_directory: str,
) -> bool:
    logging.info(f'Starting executute once [platform:{platform}-{analysis}] [input_program:{input_program}] [runtime:{runtime}]')
    benchmark_path = os.path.join(target_build_directory, input_program, f'{input_program}.cjs')

    default_row = {}
    default_row[KEY_runtime] = runtime
    default_row[KEY_platform] = platform
    default_row[KEY_analysis] = analysis
    default_row[KEY_input_program] = input_program
    default_row[KEY_timeout_amount] = timeout

    try:
        bench_run_result = subprocess.run(
            ['bash', '-c', f'node --experimental-wasm-multi-memory {benchmark_path}'],
            capture_output=True,
            text=True,
            timeout=timeout,
        )

    except subprocess.TimeoutExpired:
        logging.warning(f'[platform:{platform}-{analysis},benchmark:{input_program},runtime:{runtime}] timeout - {timeout}')
        csv_report = default_row.copy()
        csv_report[KEY_memory_usage] = None
        csv_report[KEY_completion_time] = None
        csv_report[KEY_time_unit] = None
        csv_report[KEY_exception] = False
        csv_report[KEY_exception_reason] = None
        csv_report[KEY_timeout] = True
        csv_writer.writerow(csv_report)
        return False

    if bench_run_result.returncode is not EXIT_STATUS_SUCCESS:
        identified_error = identify_error(bench_run_result.stderr)
        logging.warning(f'[platform:{platform}-{analysis},benchmark:{input_program},runtime:{runtime}] error - {identified_error}')
        csv_report = default_row.copy()
        csv_report[KEY_memory_usage] = None
        csv_report[KEY_completion_time] = None
        csv_report[KEY_time_unit] = None
        csv_report[KEY_exception] = True
        csv_report[KEY_exception_reason] = identified_error
        csv_report[KEY_timeout] = False
        csv_writer.writerow(csv_report)
        return False

    # At this point the run was a success, assert stdout reports run result

    # Blacklist lines that are of the nature 'Wasabi: hook <...> not provided by Wasabi.analysis, I will use an empty function as a fallback'
    allowed_ignore_pattern = r'Wasabi: hook [\w-]+ not provided by Wasabi.analysis, I will use an empty function as a fallback'
    bench_run_result_stdout_lines = bench_run_result.stdout.strip().split('\n')

    # Now walk over subprocess' stdout, filter 'ignore pattern'
    captured_lines: list[str] = []
    for bench_run_result_stdout_line in bench_run_result_stdout_lines:
        if len(bench_run_result_stdout_line) == 0: continue
        if re.match(allowed_ignore_pattern, bench_run_result_stdout_line): continue
        captured_lines += [bench_run_result_stdout_line]

    # assert 2 lines are kept as relevant here; one reporting performance and one reporting memory
    assert len(captured_lines) == 2

    total_time = 0
    time_unit = 'ms'
    memory_usage_after_one_run = None
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

        if pattern_match_performance is not None:
            [re_input_program, re_run, re_performance] = [pattern_match_performance.group(i) for i in [1, 2, 3]]
            assert input_program == re_input_program
            assert benchmark_report_line == f'{re_input_program} (run {re_run}): {re_performance}'
            total_time += float(re_performance)
            continue

        if pattern_match_memory is not None:
            [re_input_program, re_bytes] = [pattern_match_memory.group(i) for i in [1, 2]]
            assert input_program == re_input_program
            assert benchmark_report_line == f'{re_input_program} memory usage in bytes: {re_bytes}'
            memory_usage_after_one_run = int(re_bytes)
            continue

    assert memory_usage_after_one_run is not None, 'By now the memory usage should have been reported'
    logging.info(f'[platform:{platform}-{analysis},benchmark:{input_program},runtime:{runtime},memory_usage:{memory_usage_after_one_run}] success (took {total_time}{time_unit})')
    csv_report = default_row.copy()
    csv_report[KEY_memory_usage] = memory_usage_after_one_run
    csv_report[KEY_completion_time] = total_time
    csv_report[KEY_time_unit] = time_unit
    csv_report[KEY_exception] = False
    csv_report[KEY_exception_reason] = None
    csv_report[KEY_timeout] = False
    csv_writer.writerow(csv_report)
    return True
