# -*- coding: utf-8 -*-
import csv
import logging
import subprocess
from os import path

from allowed_stderr import filter_stderr

RUNTIME = 'Wasmtime'
RESULTS_FILE = 'results.csv'
TIMEOUT_IN_SECONDS_ONE_RUN = 1 * 60 * 5
RUNS = 30
INPUT_PROGRAMS = [
    'commanderkeen',   # CRASH ON Apply Silicon M2
    'sandspiel',       # CRASH ON Apply Silicon M2
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

INPUT_WASTRUMENTATION_ANALYSES = [
    'forward',
    'taint',
    'call-graph',
    'memory-tracing',
    'coverage-branch',
    'instruction-mix',
    'block-profiling',
    'coverage-instruction',
    'cryptominer-detection',
]

KEY_runtime = 'runtime'
KEY_platform = 'platform'
KEY_analysis = 'analysis'
KEY_input_program = 'input_program'
KEY_completion_time = 'completion_time'
KEY_time_unit = 'time_unit'
KEY_timeout = 'timeout'
KEY_timeout_amount = 'timeout_amount'
KEY_exception = 'exception'
KEY_exception_reason = 'exception_reason'

result_file_field_names = [
    KEY_runtime,
    KEY_platform,
    KEY_analysis,
    KEY_input_program,
    KEY_completion_time,
    KEY_time_unit,
    KEY_timeout,
    KEY_timeout_amount,
    KEY_exception,
    KEY_exception_reason,
]

benchmarks_containing_directory = path.join('..', 'wasmr3-python', 'working-dir', 'wasm-r3')
baseline = path.join(benchmarks_containing_directory, 'benchmarks')

def to_wastrumentation_analysis_path(analysis):
    return path.join(benchmarks_containing_directory, 'benchmarks_wastrumentation', analysis)

wastrumentation_analyses_paths = [
    ('Wastrumentation', analysis, to_wastrumentation_analysis_path(analysis))
    for
    analysis
    in
    INPUT_WASTRUMENTATION_ANALYSES
]

def execute_benchmark(platform, input_program, input_program_path, analysis, runs, timeout):
    try:
        # run benchmark & let process write to file
        process = subprocess.run(
            [
                'bash',
                '-c',
                f'cargo run --release -- --platform {platform} --input-program {input_program} --input-program-path {input_program_path} --analysis {analysis} --runs {runs}',
            ],
            timeout=timeout,
            capture_output=True,
            text=True,
        )

    except subprocess.TimeoutExpired:
        logging.warning(f'[platform:{platform},analysis:{analysis},benchmark:{input_program},runtime:{RUNTIME}] timeout - {TIMEOUT_IN_SECONDS_ONE_RUN}')

        with open(RESULTS_FILE, 'a+') as results_file:
            results_writer = csv.DictWriter(results_file, fieldnames=result_file_field_names)
            csv_values = {}

            csv_values[KEY_runtime] = RUNTIME
            csv_values[KEY_platform] = platform
            csv_values[KEY_analysis] = analysis
            csv_values[KEY_input_program] = input_program
            csv_values[KEY_completion_time] = 0
            csv_values[KEY_time_unit] = 'ms'
            csv_values[KEY_timeout] = True
            csv_values[KEY_timeout_amount] = TIMEOUT_IN_SECONDS_ONE_RUN
            csv_values[KEY_exception] = False
            csv_values[KEY_exception_reason] = None

            # Write row
            results_writer.writerow(csv_values)
            return

    print(process.stderr)
    print(process.stdout)

    stderr = filter_stderr(process.stderr)

    if stderr is not None:
        with open(RESULTS_FILE, 'a+') as results_file:
            results_writer = csv.DictWriter(results_file, fieldnames=result_file_field_names)
            csv_values = {}

            csv_values[KEY_runtime] = RUNTIME
            csv_values[KEY_platform] = platform
            csv_values[KEY_analysis] = analysis
            csv_values[KEY_input_program] = input_program
            csv_values[KEY_completion_time] = 0
            csv_values[KEY_time_unit] = 'ms'
            csv_values[KEY_timeout] = False
            csv_values[KEY_timeout_amount] = TIMEOUT_IN_SECONDS_ONE_RUN
            csv_values[KEY_exception] = True
            csv_values[KEY_exception_reason] = stderr

            # Write row
            results_writer.writerow(csv_values)

for (platform, analysis, benchmarks_directory) in [('uninstrumented', 'none', baseline)] + wastrumentation_analyses_paths:
    for input_program in INPUT_PROGRAMS:
        times_this_combination_timed_out = 0
        input_program_path = path.join(benchmarks_directory, input_program, f"{input_program}.wasm")
        assert path.exists(input_program_path), input_program_path

        # Try one run, with the timeout threshold
        execute_benchmark(platform, input_program, input_program_path, analysis, 1, TIMEOUT_IN_SECONDS_ONE_RUN)

        # Try over multiple runs
        execute_benchmark(platform, input_program, input_program_path, analysis, RUNS, RUNS * TIMEOUT_IN_SECONDS_ONE_RUN)
