#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os

from setup_workspace import setup_workspace
from fetch_polybench import fetch_polybench_benchmark_suite
from compl_polybench import compile_polybench_benchmark_suite
from execute_benchmarks import execute_benchmarks
from benchmark_datasets import BenchmarkDatasetSizes
from instrument_wasabi import instrument_wasabi

from fetch_browser import FireFoxFetcher

from config import working_directory, analyses_directory
from config import build_path_name, results_file_regular_name
from config import build_path_name_wasabi, results_file_wasabi

# fetch relevant datasets
benchmark_dataset_sizes = BenchmarkDatasetSizes()
firefox_fetcher = FireFoxFetcher()

# Setup workspace
setup_workspace()
# Fetch benchmark suite
fetch_polybench_benchmark_suite()
# Compile benchmark suite
compile_polybench_benchmark_suite(benchmark_dataset_sizes)
# Run benchmarks
execute_benchmarks(
    setup_name = '[uninstrumented]',
    browser_binary_path = firefox_fetcher.binary(),
    browser_version = firefox_fetcher.version(),
    target_build_directory = build_path_name,
    results_file_path = os.path.join(working_directory, results_file_regular_name),
    benchmark_dataset_sizes = benchmark_dataset_sizes,
)

# Instrument for Wasabi
instruction_mix_analysis_path = os.path.join(analyses_directory, 'instruction_mix.js')
instrument_wasabi(
    benchmark_dataset_sizes,
    instruction_mix_analysis_path,
    'instruction_mix'
)

# Run Wasabi benchmarks
execute_benchmarks(
    setup_name = f'[wasabi]',
    browser_binary_path = firefox_fetcher.binary(),
    browser_version = firefox_fetcher.version(),
    target_build_directory = build_path_name_wasabi,
    results_file_path = os.path.join(working_directory, results_file_wasabi),
    benchmark_dataset_sizes = benchmark_dataset_sizes,
)
