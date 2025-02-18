# -*- coding: utf-8 -*-
import os
import csv
import re
import logging
import subprocess
from os import path

KEY_runtime = 'runtime'
KEY_platform = 'platform'
KEY_analysis = 'analysis'
KEY_input_program = 'input_program'
KEY_completion_time = 'completion_time'
KEY_time_unit = 'time_unit'
KEY_timeout = 'timeout'
KEY_timeout_amount = 'timeout_amount'

result_file_field_names = [
    KEY_runtime,
    KEY_platform,
    KEY_analysis,
    KEY_input_program,
    KEY_completion_time,
    KEY_time_unit,
    KEY_timeout,
    KEY_timeout_amount,
]


TIMEOUT_IN_SECONDS = 1 * 60 * 5
TIMEOUT_THRESHOLD = 2
RUNS = 30
INPUT_PROGRAMS = [
    # "commanderkeen",   # CRASH ON Apply Silicon M2
    # "sandspiel",       # CRASH ON Apply Silicon M2
    'jqkungfu',        # 12666 ns
    'game-of-life',    # 15417 ns
    'rtexpacker',      # 18166 ns
    'factorial',       # 23500 ns
    'rtexviewer',      # 20750 ns
    'ffmpeg',          # 31084 ns
    'figma-startpage', # 50334 ns
    'parquet',         # 90250 ns
    'hydro',           # 94083 ns
    'pacalc',          # 258750 ns
    'riconpacker',     # 434000 ns
    'sqlgui',          # 649125 ns
    'jsc',             # 2401333 ns
    'boa',             # 3646083 ns
    'guiicons',        # 17659125 ns
    'rguistyler',      # 16036084 ns
    'rguilayout',      # 18067291 ns
    'rfxgen',          # 18035166 ns
    'bullet',          # 26375500 ns
    'pathfinding',     # 386305041 ns
    'funky-kart',      # 39554500 ns
    'fib',             # 2386120708 ns
    'multiplyInt',     # 2683662583 ns
    'mandelbrot',      # 3032975916 ns
    'multiplyDouble',  # 7099443958 ns
]

def write_timeout():
    ...

benchmarks_containing_directory = path.join('..', 'wasmr3-python', 'working-dir', 'wasm-r3')
baseline = path.join(benchmarks_containing_directory, 'benchmarks')
forward_wastrumentation = path.join(benchmarks_containing_directory, 'benchmarks_wastrumentation', 'forward')

for (platform, analysis, benchmarks_directory) in [
    ('uninstrumented', 'none', baseline),
    ('Wastrumentation', 'forward', forward_wastrumentation),
]:
    for input_program in INPUT_PROGRAMS:
        times_this_combination_timed_out = 0
        input_program_path = path.join(benchmarks_directory, input_program, f"{input_program}.wasm")
        assert path.exists(input_program_path), input_program_path
        for iteration in range(RUNS):
            if times_this_combination_timed_out > TIMEOUT_THRESHOLD: continue

            try:
                # run benchmark & write to file
                bench_run_result = subprocess.run(
                    [
                        'bash',
                        '-c',
                        f'cargo run --release -- --platform {platform} --input-program {input_program} --input-program-path {input_program_path} --analysis {analysis}',
                    ],
                    timeout=TIMEOUT_IN_SECONDS,
                )

            except subprocess.TimeoutExpired:
                logging.warning(f'[platform:{platform},analysis:{analysis},benchmark:{input_program},runtime:{iteration}] timeout - {TIMEOUT_IN_SECONDS}')

                with open('results.csv', 'a+') as results_file:
                    results_writer = csv.DictWriter(results_file, fieldnames=result_file_field_names)
                    csv_values = {}

                    csv_values[KEY_runtime] = 'Wasmtime'
                    csv_values[KEY_platform] = platform
                    csv_values[KEY_analysis] = analysis
                    csv_values[KEY_input_program] = input_program
                    csv_values[KEY_completion_time] = 0
                    csv_values[KEY_time_unit] = 'ms'
                    csv_values[KEY_timeout] = True
                    csv_values[KEY_timeout_amount] = TIMEOUT_IN_SECONDS

                    # Write row
                    results_writer.writerow(csv_values)

                # csv_writer.writerow(csv_values)
                times_this_combination_timed_out += 1
                continue
