# -*- coding: utf-8 -*-
# DO NOT TOUCH VARIABLES BELOW
# vvvvvvvvvvvvvvvvvvvvvvvvvvvv
ANALYSIS_BRANCHES = 'branches'
ANALYSIS_OPCODES = 'opcodes'
ANALYSIS_ICOUNT = 'icount'
ANALYSIS_GLOBALS = 'globals'
ANALYSIS_LOOPS = 'loops'
ANALYSIS_MEMSTATS = 'memstats'

ANALYSIS_TAINT = 'taint'
ANALYSIS_FORWARD = 'forward'
ANALYSIS_SAFE_HEAP = 'safe-heap'
ANALYSIS_DENAN = 'denan'
# ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
# DO NOT TOUCH VARIABLES ABOVE

# CONFIGURE ANALYSES BENCHMARK CONSTANTS
benchmark_runs = 1 # How often to rerun each NodeJS instance
NODE_BENCHMARK_RUNS = 1 # How often to rerun within a NodeJS instance

# CONFIGURE ANALYSES
configured_analyses_slower_than_forward = [
    (ANALYSIS_BRANCHES, 1.00),
    (ANALYSIS_OPCODES,  1.00),
    (ANALYSIS_ICOUNT,   1.00),
    (ANALYSIS_GLOBALS,  1.00),
    (ANALYSIS_LOOPS,    1.00),
    (ANALYSIS_FORWARD,  1.00), # <-- Do not disable
    (ANALYSIS_DENAN,    1.00),
    (ANALYSIS_SAFE_HEAP,1.00),
    (ANALYSIS_MEMSTATS, 1.00),
    (ANALYSIS_TAINT,    3.00),
]

configured_analyses = list(map(lambda ca_s: ca_s[0], configured_analyses_slower_than_forward))

# CONFIGURE INPUT PROGRAMS
input_programs_runtimes = [
    # INPUT PROGRAM   WASTRUMENTATION      WASABI
    ('rtexviewer',          0.346272,      0.436486),
    ('rtexpacker',          0.346902,      0.427286),
    ('game-of-life',        0.754667,      4.38687),
    ('jqkungfu',            1.53478,      13.9628),
    ('factorial',           1.76707,       9.94862),
    ('ffmpeg',              1.86548,       0.00000),
    ('figma-startpage',     3.49923,      18.7752),
    ('hydro',               5.33059,      36.8816),
    ('parquet',            21.7277,       94.5862),
    ('pacalc',             33.6566,      176.006),
    ('sqlgui',            124.09,       1503.17),
    ('riconpacker',       332.826,      4836.58),
    ('jsc',               554.888,         0.00),
    ('boa',              1222.82,          0.00),
    ('rguilayout',       2155.39,          0.00),
    ('rfxgen',           2268.28,       5468.58),
    ('bullet',           2442.11,       2955.46),
    ('rguistyler',       2706.76,       7396.98),
    ('guiicons',         2752.84,       5967.73),
    ('funky-kart',       2931.1,        4247.15),
    ('sandspiel',       14846.6,           0.00),
    ('pathfinding'  ,   35347.8,           0.00),
    ('commanderkeen',   40231.0,           0.00),
    ('fib',            169859.0,      214271.0),
    ('multiplyInt',    202245.0,      184175.0),
    ('mandelbrot',     213539.0,      260168.0),
    ('multiplyDouble', 246941.0,      243764.0),
]

input_programs = list(map(lambda ip_r: ip_r[0], input_programs_runtimes))
runtimes = list(map(lambda ip_r: ip_r[1], input_programs_runtimes))

def estimate_total_benchmark_time():
    total_time_ms = 0
    for (analysis, analysis_correction) in configured_analyses_slower_than_forward:
        for (input_program, wastrumentation_runtime, wasabi_runtime) in input_programs_runtimes:
            total_time_ms += wastrumentation_runtime * analysis_correction + wasabi_runtime * analysis_correction

    total_time_ms = total_time_ms * NODE_BENCHMARK_RUNS * benchmark_runs
    seconds = total_time_ms / 1000
    # source: https://stackoverflow.com/a/72979658
    (hours, seconds) = divmod(seconds, 3600)
    (minutes, seconds) = divmod(seconds, 60)

    formatted = f"ESTIMATE: {hours:02.0f} hours, {minutes:02.0f} minutes and {seconds:05.2f} seconds"
    return formatted

print(estimate_total_benchmark_time())
