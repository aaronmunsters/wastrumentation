# -*- coding: utf-8 -*-
from config import polybench_path_name, polybench_archive, polybench_link
from config import polybench_directory, working_directory
from download_unarchive import download_unarchive

import os

def fetch_polybench_benchmark_suite():
    # cd to working directory & fetch polybench benchmark suite
    os.chdir(working_directory)
    download_unarchive(polybench_path_name, polybench_archive, polybench_link)

    # In polybench_dir:
    os.chdir(polybench_directory)
    # 1. Remove irrelevant files for the benchmarks if they exist
    for file in ['AUTHORS', 'CHANGELOG', 'LICENSE.txt', 'THANKS', 'polybench.pdf']:
        if os.path.isfile(file): os.remove(file)
