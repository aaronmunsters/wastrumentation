# -*- coding: utf-8 -*-
import csv
import os
import logging

def report_code_size(
    platform: str,
    analysis: str | None,
    input_program: str,
    csv_writer: csv.DictWriter,
    target_build_directory: str,
):
    # Compute code sizes
    benchmark_directory_path = os.path.join(target_build_directory, input_program)
    binary_path = os.path.join(benchmark_directory_path, f'{input_program}.wasm')
    size_bytes = os.path.getsize(binary_path)

    # Report code sizes
    logging.info(f"[SIZE REPORT {platform: <16}-{analysis}]: PROGRAM '{input_program : <13}' - SIZE (bytes):  [{size_bytes:09}]")
    csv_writer.writerow({
        'platform': platform,
        'analysis': analysis,
        'input_program': input_program,
        'size_bytes': size_bytes,
    })
