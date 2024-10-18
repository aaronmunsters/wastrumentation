# -*- coding: utf-8 -*-
import os

# Benchmark constants
benchmark_runs = 1

# Output files
results_file_regular_name    = 'runtime-analysis-regular.csv'
results_file_wastrumentation = 'runtime-analysis-wastrumentation.csv'
results_file_wasabi          = 'runtime-analysis-wasabi.csv'

# Directory paths
analyses_directory: str = os.path.abspath('analyses')
working_directory: str = os.path.abspath('working-dir')
build_path_name: str = 'build'
build_path_name_wasabi: str = 'build_wasabi'

polybench_path_name: str = 'polybench-c'
polybench_identifier: str = 'polybench-c-4.2.1-beta'
polybench_directory: str = os.path.join(working_directory, polybench_path_name, polybench_identifier)
polybench_archive: str = f'{polybench_identifier}.tar.gz'
polybench_link: str = f'https://downloads.sourceforge.net/project/polybench/{polybench_archive}'
benchmark_list_path_in_polybench: str = 'utilities/benchmark_list'

# Dataset path
dataset_size_list_name = os.path.abspath(os.path.join(working_directory, '..', 'dataset_sizes'))

# Firefox links
firefox_link_macos: str = 'https://download-origin.cdn.mozilla.net/pub/firefox/nightly/2024/08/2024-08-05-21-59-35-mozilla-central/firefox-131.0a1.en-US.mac.dmg'
firefox_link_linux: str = 'https://download-origin.cdn.mozilla.net/pub/firefox/nightly/2024/08/2024-08-05-21-59-35-mozilla-central/firefox-131.0a1.en-US.linux-aarch64.tar.bz2'
firefox_archive: str = 'firefox.tar.bz2'
firefox_path: str = 'firefox'

# Timeout and exit status
timeout = 4000 # seconds
EXIT_STATUS_TIMEOUT = 124
