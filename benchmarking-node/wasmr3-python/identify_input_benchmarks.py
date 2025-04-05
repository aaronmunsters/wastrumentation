# -*- coding: utf-8 -*-
import os
import logging

from config import bench_suite_benchmarks_path
import input_programs_analysis_config

# https://webassembly.github.io/spec/core/binary/modules.html#binary-magic
wasm_magic_bytes = bytes([0x00, 0x61, 0x73, 0x6D])

def identify_input_benchmarks() -> dict[str, str]:
    benchmarks: dict[str, str] = {}
    for benchmark in os.listdir(bench_suite_benchmarks_path):
        # Check if `benchmark` contains candidate `.wasm` file
        bench_suite_benchmark_path = os.path.join(bench_suite_benchmarks_path, benchmark, f'{benchmark}.wasm')
        # Skip if `benchmark` contains no candidate `.wasm` file
        if not os.path.exists(bench_suite_benchmark_path):
            logging.warning(f"Identified benchmark '{benchmark}' in '{bench_suite_benchmarks_path}' contains no file '{benchmark}.wasm'?")
            logging.warning(f"Consider removing '{benchmark}' from '{bench_suite_benchmarks_path}'.")
            continue
        with open(bench_suite_benchmark_path, 'rb') as benchmark_fd:
            file_head = benchmark_fd.read(len(wasm_magic_bytes))
            if not file_head.startswith(wasm_magic_bytes):
                logging.warning(f"Identified benchmark '{benchmark}' in '{bench_suite_benchmarks_path}' contains a `{benchmark}.wasm` file, but does not start with the Wasm magic bytes?")
                logging.warning(f"Consider removing '{bench_suite_benchmark_path}' as its file extension is confusing.")
                continue
        # Skip if `benchmark` is not in enabled benchmarks
        if not benchmark in input_programs_analysis_config.input_programs:
            logging.warning(f"Identified benchmark '{benchmark}' not configured in `input_programs_analysis_config.input_programs`.")
            continue

        benchmarks[benchmark] = bench_suite_benchmark_path

    return benchmarks

# src for magic bytes check: https://stackoverflow.com/q/55587122
