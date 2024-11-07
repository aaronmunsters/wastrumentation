#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os

from identify_input_benchmarks import identify_input_benchmarks
from setup_workspace import setup_workspace
from fetch_benchmark_suite import fetch_benchmark_suite
from setup_benchmarks_regular import setup_benchmarks_regular
from setup_benchmarks_wasabi import setup_benchmarks_wasabi
from setup_benchmarks_wastrumentation import setup_benchmarks_wastrumentation
from execute_benchmarks import execute_benchmarks
from report_executes_once import report_executes_once
from report_code_size import report_code_size

from config import node_wasm_wrap_path
from config import path_results_file_regular, path_results_file_wasabi, path_results_file_wastrumentation
from config import path_code_size_results_file_regular, path_code_size_results_file_wasabi, path_code_size_results_file_wastrumentation
from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wasabi, bench_suite_benchmarks_path_wastrumentation
from config import path_executes_once_results_file_regular, path_executes_once_results_file_wasabi, path_executes_once_results_file_wastrumentation
from config import NODE_BENCHMARK_RUNS

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
forward_analysis = analysis_names_pathed[-1]
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

# Create results files!
for results_file_path in [
    path_executes_once_results_file_regular,
    path_executes_once_results_file_wasabi,
    path_executes_once_results_file_wastrumentation,
]:
    results_file = open(results_file_path, 'w')
    results_file.write('setup,runtime,input_program,executes_once,reason\n')
    results_file.flush()

# Execute all once, report if execution is a success
for benchmark in candidate_input_benchmarks.keys():
    # Report executes once
    success_uninstrumented = report_executes_once(
        setup_name = '[uninstrumented]',
        runtime_name = '[nodejs]',
        input_program = benchmark,
        target_build_directory = bench_suite_benchmarks_path,
        results_file_path = path_executes_once_results_file_regular,
    )

forward_success_runs = {}

for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in [forward_analysis]:
    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        # Run benchmarks [wasabi]
        success_on_wasabi: bool = report_executes_once(
            setup_name = f'[wasabi - {analysis_name}]',
            runtime_name = '[nodejs]',
            input_program = benchmark,
            target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
            results_file_path = path_executes_once_results_file_wasabi,
        )

        # Run benchmarks [wastrumentation]
        success_on_wastrumentation: bool = report_executes_once(
            setup_name = f'[wastrumentation - {analysis_name}]',
            runtime_name = '[nodejs]',
            input_program = benchmark,
            target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
            results_file_path = path_executes_once_results_file_wastrumentation,
        )

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

# Create results files!
for results_file_path in [
    path_code_size_results_file_regular,
    path_code_size_results_file_wasabi,
    path_code_size_results_file_wastrumentation,
]:
    results_file = open(results_file_path, 'w')
    results_file.write('setup,input_program,size_bytes\n')
    results_file.flush()

for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
    # Report code increase [uninstrumented]
    report_code_size(
        setup_name = '[uninstrumented]',
        input_program = benchmark,
        target_build_directory = bench_suite_benchmarks_path,
        results_file_path = path_code_size_results_file_regular,
    )

for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in analysis_names_pathed:
    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        if forward_success_runs[benchmark]['wasabi']:
            report_code_size(
                setup_name = f'[wasabi - {analysis_name}]',
                input_program = benchmark,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
                results_file_path = path_code_size_results_file_wasabi,
            )
        if forward_success_runs[benchmark]['wastrumentation']:
            report_code_size(
                setup_name = f'[wastrumentation - {analysis_name}]',
                input_program = benchmark,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
                results_file_path = path_code_size_results_file_wastrumentation,
            )

#####################################
###### REPORT ON THE RUN TIMES ######
#####################################

# Create results files!
for results_file_path in [
    path_results_file_regular,
    path_results_file_wasabi,
    path_results_file_wastrumentation,
]:
    results_file = open(results_file_path, 'w')
    results_file.write('setup,runtime,input_program,runtime_iteration,performance,time-unit\n')
    results_file.flush()

for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
    execute_benchmarks(
        setup_name = '[uninstrumented]',
        runtime_name = '[nodejs]',
        input_program = benchmark,
        target_build_directory = bench_suite_benchmarks_path,
        results_file_path = path_results_file_regular,
    )

for (analysis_name, wasabi_analysis_path, wastrumentation_analysis_path, wasabi_hooks, wastrumentation_hooks) in analysis_names_pathed:
    for (benchmark, benchmark_path) in candidate_input_benchmarks.items():
        if forward_success_runs[benchmark]['wasabi']:
            execute_benchmarks(
                setup_name = f'[wasabi - {analysis_name}]',
                runtime_name = '[nodejs]',
                input_program = benchmark,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wasabi, analysis_name),
                results_file_path = path_results_file_wasabi,
            )

        if forward_success_runs[benchmark]['wastrumentation']:
            execute_benchmarks(
                setup_name = f'[wastrumentation - {analysis_name}]',
                runtime_name = '[nodejs]',
                input_program = benchmark,
                target_build_directory = os.path.join(bench_suite_benchmarks_path_wastrumentation, analysis_name),
                results_file_path = path_results_file_wastrumentation,
            )
