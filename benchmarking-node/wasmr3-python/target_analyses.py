# -*- coding: utf-8 -*-
import os

# type AnalysisName = str
# type WasabiHooks = list[str]
# type WastrumentationHooks = list[str]

# type AnalysisPathWasabi          = str
# type AnalysisPathWastrumentation = str

# type: [AnalysisName, WasabiHooks, WastrumentationHooks][]
analysis_names_primitive = [
    [
        'instruction-mix',
        #                                                                                                                                                                                                                                                                                                                  Note: begin is covered in many _pre traps
        # ❌❌   ❌❌❌❌❌❌❌    ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅
        ['nop', 'unreachable', 'if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'begin', 'return'],
        [                      'if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'block-post', 'loop-pre', 'loop-post'],
    ],
    [
        'coverage-instruction',
        #                                                                                                                                                                                                                                                                                                                  Note: begin is covered in many _pre traps
        # ❌❌   ❌❌❌❌❌❌❌    ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅   ✅✅✅✅    ✅✅✅        ✅✅✅✅✅       ✅✅✅   ✅✅✅✅    ✅✅✅✅✅✅✅    ✅✅✅✅✅✅     ✅✅     ✅✅✅✅   ✅✅✅    ✅✅✅    ✅✅✅    ✅✅✅✅   ✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅  ✅✅✅✅   ✅✅✅
        ['nop', 'unreachable', 'if',                                                           'br',     'br_if',     'br_table',     'drop', 'select', 'memory_size', 'memory_grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call',                                                             'const', 'begin', 'return'],
        [                      'if-then', 'if-then-post', 'if-then-else', 'if-then-else-post', 'branch', 'branch-if', 'branch-table', 'drop', 'select', 'memory-size', 'memory-grow', 'unary', 'binary', 'load', 'store', 'local', 'global', 'call-pre', 'call-post', 'call-indirect-pre', 'call-indirect-post', 'const', 'return', 'block-pre', 'block-post', 'loop-pre', 'loop-post'],
    ],
    [
        'coverage-branch',
        # ✅                         ✅✅✅         ✅✅✅✅✅       ✅✅✅✅
        ['if',                      'br_if',     'br_table',     'select'],
        ['if-then-else', 'if-then', 'branch-if', 'branch-table', 'select'],
    ],
    [
        'call-graph',
        # ✅
        ['call'],
        ['call-pre'],
    ],
    [
        'memory-tracing',
        # ✅✅✅   ✅✅✅
        ['load', 'store'],
        ['load', 'store'],
    ],
    [
        'cryptominer-detection',
        # ✅✅✅   ✅✅✅
        ['binary'],
        ['binary'],
    ],
    [
        'block-profiling',
        # ✅✅✅   ✅✅✅
        ['begin'],
        ['if-then', 'if-then-else', 'branch', 'branch-if', 'branch-table', 'call-pre', 'call-indirect-pre', 'block-pre', 'loop-pre'],
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
]
