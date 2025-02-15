#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os

import csv
import logging
from identify_input_benchmarks import identify_input_benchmarks
from setup_workspace import setup_workspace
from fetch_benchmark_suite import fetch_benchmark_suite
from setup_benchmarks_regular import setup_benchmarks_regular
from setup_benchmarks_wasabi import setup_benchmarks_wasabi
from setup_benchmarks_wastrumentation import setup_benchmarks_wastrumentation
from execute_benchmarks import execute_benchmarks
from report_executes_once import report_executes_once, forward_success_runs_for
from report_code_size import report_code_size

from config import node_wasm_wrap_path, path_executes_once, path_code_size, path_execution_bench
from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wasabi, bench_suite_benchmarks_path_wastrumentation
from input_programs_analysis_config import NODE_BENCHMARK_RUNS
from config import executes_once_field_names, execution_bench_field_names, code_size_field_names
from config import KEY_runtime, KEY_platform, KEY_analysis, KEY_input_program, KEY_memory_usage, KEY_completion_time, KEY_time_unit, KEY_exception, KEY_exception_reason, KEY_timeout, KEY_timeout_amount
from config import timeout

from target_analyses import analysis_names_pathed
from util import parse_boolean

path_RERUN_executes_once = path_executes_once + '.rerun.csv'


# Setup workspace & Fetch benchmark suite
setup_workspace()
fetch_benchmark_suite()

# Setup benchmarks regular
intra_vm_benchmark_runs = 1
candidate_input_benchmarks = identify_input_benchmarks()

# FIXME: hardcoded case
forward_analysis = analysis_names_pathed[-1]
(forward_analysis_name, _, _, _, _) = forward_analysis
assert forward_analysis_name == 'forward', f"Assumption that last in analysis_names_pathed is 'forward' analysis violated"

###############################################################
###### REPORT SUCCESS RUNS FOR INSTRUMENTATION PLATFORMS ######
###############################################################

def platform_to_bench_path(platform: str) -> str:
    path = {
        'Wasabi': bench_suite_benchmarks_path_wasabi,
        'uninstrumented': bench_suite_benchmarks_path,
        'Wastrumentation': bench_suite_benchmarks_path_wastrumentation,
    }[platform]
    return path

executes_once_data = None
with open(path_executes_once, 'r') as file:
    executes_once_data = file.read()

# Attempt to rerun 'once' executions
with open(path_RERUN_executes_once, 'w') as executes_once_file:
    # Open results csv writer
    executes_once_writer = csv.DictWriter(executes_once_file, fieldnames=executes_once_field_names)
    executes_once_writer.writeheader()

    # Open previous run reader
    executes_once_reader = csv.DictReader(executes_once_data.splitlines(), fieldnames=executes_once_field_names)
    next(executes_once_reader, None)  # skip the headers - https://stackoverflow.com/a/14257599
    for executes_once_measure in executes_once_reader:
        timed_out = parse_boolean(executes_once_measure[KEY_timeout])
        if not timed_out:
            logging.info('Skipping a rerun ...')
            executes_once_writer.writerow(executes_once_measure)
            executes_once_file.flush()
            continue
        else:
            previous_run_timed_out_seconds = int(executes_once_measure[KEY_timeout_amount])
            if previous_run_timed_out_seconds >= timeout:
                logging.warning(f'Not rerunning {executes_once_measure} since its timeout {previous_run_timed_out_seconds} is higher than current timeout {timeout}')
                executes_once_writer.writerow(executes_once_measure)
                executes_once_file.flush()
                continue
            # Else perform run
            analysis_name = executes_once_measure[KEY_analysis]
            platform = executes_once_measure[KEY_platform]
            target_build_directory = os.path.join(platform_to_bench_path(executes_once_measure[KEY_platform]), analysis_name)
            new_execute_once_success = report_executes_once(
                runtime = executes_once_measure[KEY_runtime],
                platform = platform,
                analysis = analysis_name,
                input_program = executes_once_measure[KEY_input_program],
                csv_writer = executes_once_writer,
                target_build_directory = target_build_directory)
            if new_execute_once_success:
                logging.info(f'Rerunning for {analysis_name} on {platform} was a success!')
            executes_once_file.flush()

##########################################################
###### SETUP OF CODE (S2S transform) FOR BENCHMARKS ######
##########################################################

candidate_input_benchmarks = identify_input_benchmarks()
input_programs = list(candidate_input_benchmarks.keys())

# Setup benchmarks regular - SKIPPED

forward_success_runs = forward_success_runs_for(path_RERUN_executes_once)
for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in analysis_names_pathed:
    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        if forward_success_runs[benchmark]['wasabi']:
            # Setup benchmarks wasabi
            setup_benchmarks_wasabi(
                node_wasm_wrap_path,
                benchmark,
                benchmark_path,
                analysis_name,
                wasabi_analysis_path,
                wasabi_hooks,
                NODE_BENCHMARK_RUNS,
            )

        if forward_success_runs[benchmark]['wastrumentation']:
            # Setup benchmarks wastrumentation
            setup_benchmarks_wastrumentation(
                node_wasm_wrap_path,
                benchmark,
                benchmark_path,
                analysis_name,
                wastrumentation_analysis_path,
                wastrumentation_hooks,
                NODE_BENCHMARK_RUNS,
            )

######################################
###### REPORT ON THE CODE SIZES ######
######################################
with open(path_code_size, 'w') as code_size_file:
    code_size_writer = csv.DictWriter(code_size_file, fieldnames=code_size_field_names)
    code_size_writer.writeheader()

    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        # Report code increase [uninstrumented]
        report_code_size(
            platform = 'uninstrumented',
            analysis = None,
            input_program = benchmark,
            csv_writer = code_size_writer,
            target_build_directory = bench_suite_benchmarks_path,
        )
        code_size_file.flush()

    for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in analysis_names_pathed:
        for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
            if forward_success_runs[benchmark]['wasabi']:
                report_code_size(
                    platform = 'Wasabi',
                    analysis = analysis_name,
                    input_program = benchmark,
                    csv_writer = code_size_writer,
                    target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
                )
                code_size_file.flush()
            if forward_success_runs[benchmark]['wastrumentation']:
                report_code_size(
                    platform = 'Wastrumentation',
                    analysis = analysis_name,
                    input_program = benchmark,
                    csv_writer = code_size_writer,
                    target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
                )
                code_size_file.flush()

#####################################
###### REPORT ON THE RUN TIMES ######
#####################################

with open(path_execution_bench, 'w') as execution_bench_file:
    execution_bench_writer = csv.DictWriter(execution_bench_file, fieldnames=execution_bench_field_names)
    execution_bench_writer.writeheader()

    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        execute_benchmarks(
            runtime = 'NodeJS',
            platform = 'uninstrumented',
            analysis = None,
            input_program = benchmark,
            csv_writer = execution_bench_writer,
            target_build_directory = bench_suite_benchmarks_path,
        )

    for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in analysis_names_pathed:
        for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
            if forward_success_runs[benchmark]['wasabi']:
                execute_benchmarks(
                    runtime = 'NodeJS',
                    platform = 'Wasabi',
                    analysis = analysis_name,
                    input_program = benchmark,
                    csv_writer = execution_bench_writer,
                    target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
                )

            if forward_success_runs[benchmark]['wastrumentation']:
                execute_benchmarks(
                    runtime = 'NodeJS',
                    platform = 'Wastrumentation',
                    analysis = analysis_name,
                    input_program = benchmark,
                    csv_writer = execution_bench_writer,
                    target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
                )
