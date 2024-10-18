# -*- coding: utf-8 -*-
import subprocess
import os

from config import working_directory
from config import bench_suite_uri, bench_suite_commit, bench_suite_path, bench_suite_benchmarks_path

def fetch_benchmark_suite():
    # cd to working directory
    os.chdir(working_directory)
    # fetch polybench benchmark suite if not present
    if not os.path.exists(bench_suite_uri):
        subprocess.run(['bash', '-c', f'git clone {bench_suite_uri}'])
    # cd to bench directory
    os.chdir(bench_suite_path)
    subprocess.run(['bash', '-c', f'git checkout {bench_suite_commit}'])

    # assert that:
    # - bench dir exists & is non-empty
    assert os.path.exists(bench_suite_benchmarks_path)
    assert len(os.listdir(bench_suite_benchmarks_path)) > 0
    # - current commit is indeed `bench_suite_commit`
    current_commit = subprocess.run(
        ['bash', '-c', f'git rev-parse HEAD'],
        capture_output=True,
        text=True,
    ).stdout.strip()
    assert current_commit == bench_suite_commit
