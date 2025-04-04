# -*- coding: utf-8 -*-
import os

from input_programs_analysis_config import configured_analyses

# type AnalysisName = str
# type WasabiHooks = list[str]
# type WastrumentationHooks = list[str]

# type AnalysisPathWasabi          = str
# type AnalysisPathWastrumentation = str

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

# type: [AnalysisName, WasabiHooks, WastrumentationHooks][]
analysis_names_primitive = [
    [
        ANALYSIS_BRANCHES,
        # ✅                         ✅✅✅         ✅✅✅✅✅
        ['if',                      'br_if',     'br_table'],
        ['if-then-else', 'if-then', 'branch-if', 'branch-table'],
    ],
    [
        ANALYSIS_OPCODES,
        # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
        ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
        ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'loop-pre', 'block-post', 'loop-post'],
    ],
    [
        ANALYSIS_ICOUNT,
        # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
        ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
        ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'loop-pre', 'block-post', 'loop-post'],
    ],
    [
        ANALYSIS_GLOBALS,
        # ✅✅✅✅✅
        ['global'],
        ['global'],
    ],
    [
        ANALYSIS_LOOPS,
        # ✅✅✅✅✅
        ['begin'],
        ['loop-pre'],
    ],
    [
        ANALYSIS_MEMSTATS,
        # ✅✅    ✅✅✅✅
        ['load', 'store'],
        ['load', 'store'],

    ],
    [
        ANALYSIS_FORWARD,
        # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
        ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
        ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'loop-pre', 'block-post', 'loop-post'],
    ],
    # [
    #     ANALYSIS_INSTRUCTION_MIX, # not included for Wasabi: nop, unreachable (neither for Wastrumentation)
    #     # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
    #     ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
    #     ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'block-post', 'loop-pre', 'loop-post'],
    # ],
    # [
    #     ANALYSIS_COVERAGE_BRANCH, # not included for Wasabi: nop, unreachable (neither for Wastrumentation)
    #     # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
    #     ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
    #     ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'block-post', 'loop-pre', 'loop-post'],
    # ],
    # [
    #     ANALYSIS_COVERAGE_BRANCH,
    #     # ✅                         ✅✅✅         ✅✅✅✅✅       ✅✅✅✅
    #     ['if',                      'br_if',     'br_table',     'select'],
    #     ['if-then-else', 'if-then', 'branch-if', 'branch-table', 'select'],
    # ],
    # [
    #     ANALYSIS_CALL_GRAPH,
    #     # ✅
    #     ['call'],
    #     ['call-pre'],
    # ],
    # [
    #     ANALYSIS_MEMORY_TRACING,
    #     # ✅✅✅   ✅✅✅
    #     ['load', 'store'],
    #     ['load', 'store'],
    # ],
    # [
    #     ANALYSIS_CRYPTOMINER_DETECTION,
    #     # ✅✅✅   ✅✅✅
    #     ['binary'],
    #     ['binary'],
    # ],
    # [
    #     ANALYSIS_BLOCK_PROFILING,
    #     # ✅✅✅   ✅✅✅
    #     ['begin'],
    #     ['call-pre', 'call-indirect-pre', 'block-pre', 'loop-pre'],
    # ],
    [
        ANALYSIS_SAFE_HEAP,
        # ✅✅✅   ✅✅✅
        [],
        ['load', 'store',],
    ],
    [
        ANALYSIS_DENAN,
        [],
        ['generic-apply', 'unary', 'binary', 'const', 'local', 'global', 'load', 'store'],
    ],
    [
        ANALYSIS_TAINT,
        # ❌❌❌❌❌❌❌❌  ❌❌   ❌❌❌❌❌❌❌    ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
        [               'nop', 'unreachable', 'if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
        ['generic-apply',                     'if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'block-post', 'loop-pre', 'loop-post'],
    ],
    [
        ANALYSIS_FORWARD,
        # ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅   # Note: begin is covered in many _pre traps
        ['if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'return', 'begin', 'end'],
        ['if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'loop-pre', 'block-post', 'loop-post'],
    ],
]

# type: (AnalysisName, AnalysisPathWasabi, AnalysisPathWastrumentation, WasabiHooks, WastrumentationHooks)[]
analysis_names_pathed = [
    [
        analysis_name,
        os.path.abspath(f'../input-analyses/javascript/{analysis_name}.cjs'),
        os.path.abspath(f'../input-analyses/rust/{analysis_name}'),
        wasabi_hooks,
        wastrumentation_hooks
    ]
    for
    analysis_name, wasabi_hooks, wastrumentation_hooks
    in
    analysis_names_primitive
    if
    analysis_name in configured_analyses
]
