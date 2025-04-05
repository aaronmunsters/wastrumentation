# -*- coding: utf-8 -*-
import re

known_success_patterns: list[str] = [
    # Finished `release` compilation
    r'[\s]*Finished `release` profile \[optimized\] target\(s\) in [\d+][.[\d+]+]?[s|ms]',
    # Running cargo
    r'[\s]*Running `target\/release\/wasmtime-benchmarks --platform [\w]+ --input-program [\w-]+ --input-program-path [-\.\/\w]+\.wasm --analysis [\w-]+ --runs \d+',

    # Allowed, not really a 'success'
    r'note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace',
]

known_failure_patterns: list[tuple[str, str]] = [
    ('Wasmtime assertion failure',  r'assertion failed: \(label_offset - offset\) <= kind.max_pos_range\(\)'),
]

def filter_stderr(stderr: str) -> str | None:
    # split stderr into lines
    stderr_lines = stderr.splitlines()
    # filter out empty lines, strip each line
    stderr_lines = filter(lambda l: len(l) != 0, [line.strip() for line in stderr_lines])
    # filter out known patterns
    stderr_lines = [line for line in stderr_lines if not any(map(lambda known_pattern: re.search(known_pattern, line), known_success_patterns))]

    if len(stderr_lines) == 0: return None

    for (known_reason, known_failure_pattern) in known_failure_patterns:
        for line in stderr_lines:
            if re.search(known_failure_pattern, line) is not None:
                return known_reason
    return 'Unknown reason::' + '-'.join(stderr_lines)
