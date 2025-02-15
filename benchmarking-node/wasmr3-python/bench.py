#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os

import csv
from identify_input_benchmarks import identify_input_benchmarks
from setup_workspace import setup_workspace
from fetch_benchmark_suite import fetch_benchmark_suite
from setup_benchmarks_regular import setup_benchmarks_regular
from setup_benchmarks_wasabi import setup_benchmarks_wasabi
from setup_benchmarks_wastrumentation import setup_benchmarks_wastrumentation
from execute_benchmarks import execute_benchmarks
from report_executes_once import report_executes_once
from report_code_size import report_code_size

from config import node_wasm_wrap_path, path_executes_once, path_code_size, path_execution_bench
from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wasabi, bench_suite_benchmarks_path_wastrumentation
from input_programs_analysis_config import NODE_BENCHMARK_RUNS, ANALYSIS_FORWARD

from config import code_size_field_names
from config import executes_once_field_names
from config import execution_bench_field_names

from target_analyses import analysis_names_pathed

# Setup workspace & Fetch benchmark suite
setup_workspace()
fetch_benchmark_suite()

###########################################
###### SETUP OF CODE (S2S transform) ######
###########################################

# Setup benchmarks regular
intra_vm_benchmark_runs = 1
candidate_input_benchmarks = identify_input_benchmarks()
setup_benchmarks_regular(node_wasm_wrap_path, candidate_input_benchmarks, intra_vm_benchmark_runs)

# Instrument the benchmarks, only for `forward analysis`
# FIXME: hardcoded case
forward_analysis = next(filter(lambda analysis_name_pathed: analysis_name_pathed[0] == ANALYSIS_FORWARD, analysis_names_pathed))
(forward_analysis_name, _, _, _, _) = forward_analysis
assert forward_analysis_name == 'forward', f"Assumption that last in analysis_names_pathed is 'forward' analysis violated"

for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in [forward_analysis]:
    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        # Setup benchmarks wasabi
        setup_benchmarks_wasabi(
            node_wasm_wrap_path,
            benchmark,
            benchmark_path,
            analysis_name,
            wasabi_analysis_path,
            wasabi_hooks,
            intra_vm_benchmark_runs,
        )
        # Setup benchmarks wastrumentation
        setup_benchmarks_wastrumentation(
            node_wasm_wrap_path,
            benchmark,
            benchmark_path,
            analysis_name,
            wastrumentation_analysis_path,
            wastrumentation_hooks,
            intra_vm_benchmark_runs,
        )

###############################################################
###### REPORT SUCCESS RUNS FOR INSTRUMENTATION PLATFORMS ######
###############################################################

forward_success_runs = {}

with open(path_executes_once, 'w') as executes_once_file:
    executes_once_writer = csv.DictWriter(executes_once_file, fieldnames=executes_once_field_names)
    executes_once_writer.writeheader()

    # Execute all once, report if execution is a success
    for benchmark in candidate_input_benchmarks.keys():
        # Report executes once
        success_uninstrumented = report_executes_once(
            runtime = 'NodeJS',
            platform = 'uninstrumented',
            analysis = None,
            input_program = benchmark,
            csv_writer = executes_once_writer,
            target_build_directory = bench_suite_benchmarks_path,
        )
        executes_once_file.flush()

    for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in [forward_analysis]:
        for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
            # Run benchmarks [wasabi]
            success_on_wasabi: bool = report_executes_once(
                runtime = 'NodeJS',
                platform = 'Wasabi',
                analysis = analysis_name,
                input_program = benchmark,
                csv_writer = executes_once_writer,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
            )
            executes_once_file.flush()

            # Run benchmarks [wastrumentation]
            success_on_wastrumentation: bool = report_executes_once(
                runtime = 'NodeJS',
                platform = 'Wastrumentation',
                analysis = analysis_name,
                input_program = benchmark,
                csv_writer = executes_once_writer,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
            )
            executes_once_file.flush()

            if benchmark not in forward_success_runs: forward_success_runs[benchmark] = {}
            forward_success_runs[benchmark]['wasabi'] = success_on_wasabi
            forward_success_runs[benchmark]['wastrumentation'] = success_on_wastrumentation

##########################################################
###### SETUP OF CODE (S2S transform) FOR BENCHMARKS ######
##########################################################

candidate_input_benchmarks = identify_input_benchmarks()
input_programs = list(candidate_input_benchmarks.keys())

# Setup benchmarks regular
setup_benchmarks_regular(node_wasm_wrap_path, candidate_input_benchmarks, NODE_BENCHMARK_RUNS)

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
