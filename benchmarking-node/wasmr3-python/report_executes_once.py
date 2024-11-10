# -*- coding: utf-8 -*-
import os
import re
import logging
import subprocess

from allowed_failures import identify_error
from config import timeout, EXIT_STATUS_SUCCESS

def report_executes_once(
    setup_name: str,
    runtime_name: str,
    input_program: str,
    target_build_directory: str,
    results_file_path: str,
) -> bool:
    with open(results_file_path, 'a') as results_file:
        logging.info(f'Starting executute once [setup:{setup_name}] [benchmark:{input_program}] [runtime:{runtime_name}]')
        benchmark_path = os.path.join(target_build_directory, input_program, f'{input_program}.cjs')

        try:
            bench_run_result = subprocess.run(
                ['bash', '-c', f'node --experimental-wasm-multi-memory {benchmark_path}'],
                capture_output=True,
                text=True,
                timeout=timeout,
            )

        except subprocess.TimeoutExpired:
            logging.warning(f'[setup:{setup_name},benchmark:{input_program},runtime:{runtime_name}] timeout - {timeout}')
            #                     setup,         runtime_name,    input_program,    executes_once,time,unit, reason
            results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","0","{timeout}","s","timeout"\n')
            results_file.flush()
            return False

        if bench_run_result.returncode is not EXIT_STATUS_SUCCESS:
            identified_error = identify_error(bench_run_result.stderr)
            logging.warning(f'[setup:{setup_name},benchmark:{input_program},runtime:{runtime_name}] error - {identified_error}')
            #                     setup,         runtime_name,    input_program,    executes_once,time,unit,reason
            results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","0","0","ms","error - {identified_error}"\n')
            results_file.flush()
            return False

        # At this point the run was a success, assert stdout reports run result

        # Blacklist lines that are of the nature 'Wasabi: hook <...> not provided by Wasabi.analysis, I will use an empty function as a fallback'
        allowed_ignore_pattern = r'Wasabi: hook [\w-]+ not provided by Wasabi.analysis, I will use an empty function as a fallback'
        bench_run_result_stdout_lines = bench_run_result.stdout.strip().split('\n')

        # Now walk over subprocess' stdout, filter 'ignore pattern'
        captured_lines = []
        for bench_run_result_stdout_line in bench_run_result_stdout_lines:
            if len(bench_run_result_stdout_line) == 0: continue
            if re.match(allowed_ignore_pattern, bench_run_result_stdout_line): continue
            captured_lines += [bench_run_result_stdout_line]

        # assert exactly 'benchmark_runs' amount of lines are kept as 'relevant' here!
        assert len(captured_lines) == 1

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

        #                     setup,         runtime_name,    input_program,    executes_once,time, unit,        reason
        results_file.write(f'"{setup_name}","{runtime_name}","{input_program}","1","{total_time}","{time_unit}","success"\n')
        results_file.flush()
        logging.info(f'[setup:{setup_name},benchmark:{input_program},runtime:{runtime_name}] success (took {total_time}{time_unit})')
        return True
