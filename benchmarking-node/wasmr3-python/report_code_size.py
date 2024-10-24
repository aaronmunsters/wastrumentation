# -*- coding: utf-8 -*-
import os

from config import bench_suite_benchmarks_path

def report_code_size(
    setup_name: str,
    input_programs: list[str],
    target_build_directory: str,
    results_file_path: str,
):
    os.chdir(bench_suite_benchmarks_path)
    results_file = open(results_file_path, 'a')

    # Report code sizes
    for count, input_program in enumerate(input_programs):
        benchmark_directory_path = os.path.join(target_build_directory, input_program)
        binary_path = os.path.join(benchmark_directory_path, f'{input_program}.wasm')
        size_bytes = os.path.getsize(binary_path)

        #                      setup_name✅,  input_program✅,  size_bytes✅\n'
        results_file.write(f'"{setup_name}","{input_program}","{size_bytes}"\n')

        print(
            f"[SIZE REPORT {setup_name: <41}]: PROGRAM [{count+1 : <3}/{len(input_programs)}] '{input_program : <13}' - SIZE (bytes):  [{size_bytes:09}]"
        )

    results_file.flush()
