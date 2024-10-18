# -*- coding: utf-8 -*-
import os
import subprocess

from config import build_path_name, benchmark_list_path_in_polybench, polybench_directory
from benchmark_datasets import BenchmarkDatasetSizes

def compile_polybench_benchmark_suite(benchmark_dataset_sizes: BenchmarkDatasetSizes):
    # cd to polybench_directory & create build directory
    os.chdir(polybench_directory)
    os.makedirs(build_path_name, exist_ok=True)

    benchmark_list = open(benchmark_list_path_in_polybench)
    for sourcefile in benchmark_list:
        sourcefile = sourcefile.strip()
        sourcedir = os.path.dirname(sourcefile)
        name = os.path.splitext(os.path.basename(sourcefile))[0]

        # Get dataset size for program
        dataset_size = benchmark_dataset_sizes.benchmark_dataset_sizes.get(name)

        # Skip if no associated dataset size
        if not dataset_size:
            print(f'WARNING: the program {name} has no associated dataset size ... skipping!')
            continue

        # Skip if already compiled
        if all(os.path.isfile(f'build/{name}{ext}') for ext in ['.wasm', '.js', '.html']):
            print(f'[already compiled] skipping compilation {name}; {dataset_size}')
            continue

        print(f'Compiling {name} (for {dataset_size})')
        subprocess.run([
            'emcc', '-O3', '-I', 'utilities', '-I', sourcedir,
            'utilities/polybench.c', sourcefile,
            '-s', 'ALLOW_MEMORY_GROWTH=1', '--emrun',
            '-DPOLYBENCH_TIME', f'-D{dataset_size}',
            '-o', f'build/{name}.html'
        ])
