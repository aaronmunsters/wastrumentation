# -*- coding: utf-8 -*-
import re

known_patterns: list[str] = [
    r'Node\.js v(\d+)\.(\d+)\.(\d+)', # NodeJS Version Pattern
    r'node:internal/process/promises:(\d+)', # NodeJS module reference for promises
    r'triggerUncaughtException\(err, true /\* fromPromise \*/\);', # NodeJS promise failure
    r'\^', # Error pointer
]

known_errors: list[tuple[str, str]] = [
    ('Invalid data segment',  r'\[CompileError: WebAssembly\.instantiate\(\): Compiling function #(\d+)(:"[a-z_]+")? failed: invalid data segment index: (\d+) @\+(\d+)\]'),
    ('Local count too large', r'\[CompileError: WebAssembly\.instantiate\(\): Compiling function #(\d+)(:"[a-z0-9_]+")? failed: local count too large @\+(\d+)\]'),
]

def identify_error(stderr: str) -> str:
    # split stderr into lines
    stderr_lines = stderr.split('\n')
    # filter out empty lines, strip each line
    stderr_lines = filter(lambda l: len(l) != 0, [line.strip() for line in stderr_lines])
    # filter out known patterns
    stderr_lines = [line for line in stderr_lines if not any(map(lambda known_pattern: re.search(known_pattern, line), known_patterns))]

    assert len(stderr_lines) == 1, f'Too many error lines: {stderr_lines}'
    relevant_error_report = stderr_lines[0]

    for (known_error_kind, known_error_pattern) in known_errors:
        error_match = re.match(known_error_pattern, relevant_error_report)
        if error_match:
            return known_error_kind

    raise Exception(f"Could not identify error '{relevant_error_report}' ... this is a bug!")

## Test code:
# known_failure = []
# example = """
#
# node:internal/process/promises:391
#     triggerUncaughtException(err, true /* fromPromise */);
#     ^
#
# [CompileError: WebAssembly.instantiate(): Compiling function #123:"r3_main" failed: local count too large @+4077]
#
# Node.js v22.5.1
#
# """
