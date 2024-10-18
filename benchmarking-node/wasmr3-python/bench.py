#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os

from setup_workspace import setup_workspace
from fetch_benchmark_suite import fetch_benchmark_suite
from setup_benchmarks_regular import setup_benchmarks_regular
from setup_benchmarks_wasabi import setup_benchmarks_wasabi
from setup_benchmarks_wastrumentation import setup_benchmarks_wastrumentation
from execute_benchmarks import execute_benchmarks

from config import node_wasm_wrap_path
from config import path_results_file_regular, path_results_file_wasabi, path_results_file_wastrumentation
from config import bench_suite_benchmarks_path, bench_suite_benchmarks_path_wasabi, bench_suite_benchmarks_path_wastrumentation

# ✅ success
success_on_both = [
    'factorial',
    'figma-startpage',
    'game-of-life',
    'hydro',
    'rtexviewer',
    'jqkungfu',
    'parquet',
    'rtexpacker',
]

# ❌ 🙄 crash on wasabi, but not on wastrumentation
success_on_wasabi_but_not_on_wastrumentation = [
    'boa',
    'ffmpeg',
    'pathfinding',
    'sandspiel',
]

# ❌ crash on wasabi/wastrumentation
crash_on_wasabi_or_wastrumentation = [
    'commanderkeen',
    'jsc',
    'pacalc',
    'rguilayout',
    'riconpacker',
    'bullet',
    'sqlgui',
    'funky-kart',
    'guiicons',
    'rfxgen',
    'rguistyler',

]

# ⏱️ too slow
too_slow = [
    'multiplyDouble',
    'fib',
    'mandelbrot',
    'multiplyInt',
]

input_programs = success_on_both

wasabi_analysis_path = os.path.abspath('../input-analyses/javascript/instruction-mix.cjs')
wastrumentation_analysis_path = os.path.abspath('../input-analyses/rust/instruction-mix')

# Setup workspace
setup_workspace()
# Fetch benchmark suite
fetch_benchmark_suite()


# Setup benchmarks regular
setup_benchmarks_regular(node_wasm_wrap_path, input_programs)
# Setup benchmarks wasabi
setup_benchmarks_wasabi(node_wasm_wrap_path, input_programs, wasabi_analysis_path)
# Setup benchmarks wastrumentation
setup_benchmarks_wastrumentation(node_wasm_wrap_path, input_programs, wastrumentation_analysis_path)

# Run benchmarks [uninstrumented]
execute_benchmarks(
    setup_name = '[uninstrumented]',
    runtime_name = '[nodejs]',
    input_programs = input_programs,
    target_build_directory = bench_suite_benchmarks_path,
    results_file_path = path_results_file_regular,
)

# Run benchmarks [wasabi]
execute_benchmarks(
    setup_name = '[wasabi]',
    runtime_name = '[nodejs]',
    input_programs = input_programs,
    target_build_directory = bench_suite_benchmarks_path_wasabi,
    results_file_path = path_results_file_wasabi,
)

# Run benchmarks [wastrumentation]
execute_benchmarks(
    setup_name = '[wastrumentation]',
    runtime_name = '[nodejs]',
    input_programs = input_programs,
    target_build_directory = bench_suite_benchmarks_path_wastrumentation,
    results_file_path = path_results_file_wastrumentation,
)
