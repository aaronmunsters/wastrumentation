# -*- coding: utf-8 -*-
import os
import subprocess
import shutil
import re

from config import build_path_name, polybench_directory
from config import build_path_name_wasabi
from benchmark_datasets import BenchmarkDatasetSizes

def instrument_wasabi(
    benchmark_dataset_sizes: BenchmarkDatasetSizes,
    analysis_src_path: str,
    analysis_src_name: str,
):
    # cd to polybench_directory & assert build directory exists
    os.chdir(polybench_directory)
    if not os.path.exists(build_path_name):
        print(f'Warning, the build directory {build_path_name} does not exist. Did compilation happen before?')
        print('Aborting out of precaution...')
        exit(0)

    # mkdir for instrumented variants
    os.makedirs(build_path_name_wasabi, exist_ok=True)

    shutil.copy(analysis_src_path, f'{build_path_name_wasabi}/analysis.js')

    for benchmark in benchmark_dataset_sizes.benchmark_dataset_sizes.keys():
        # Require all files exist
        if not all(os.path.isfile(f'{build_path_name}/{benchmark}{ext}') for ext in ['.wasm', '.js', '.html']):
            print(f'[compiled benchmark missing] one of {benchmark}.[wasm|js|html] in {build_path_name} is missing')
            print('Aborting out of precaution...')
            exit(0)
            continue

        shutil.copy(f'{build_path_name}/{benchmark}.wasm', f'{build_path_name_wasabi}/{benchmark}.wasm')
        shutil.copy(f'{build_path_name}/{benchmark}.html', f'{build_path_name_wasabi}/{benchmark}.html')
        shutil.copy(f'{build_path_name}/{benchmark}.js', f'{build_path_name_wasabi}/{benchmark}.js')

        # The following command:
        #
        #   wasabi --output-dir [dir] [<input>.wasm]
        #
        # will output a `<input>.wasabi.js` file and
        # an instrumented `<input>.wasm` file in the
        # output directory [dir]
        subprocess.run([
            'bash', '-c', f"""                          \
            wasabi                                      \
                --output-dir "{build_path_name_wasabi}"  \
                "{f"{build_path_name}/{benchmark}.wasm"}"
            """
        ])

        # Now, inject the instrumentation script from Wasabi
        source_html_path = f'{build_path_name}/{benchmark}.html'
        destin_html_path = f'{build_path_name_wasabi}/{benchmark}.html'
        pattern = fr'<script\s+async\s+src={benchmark}\.js></script>'
        replacement = f"""                                    \
            <script async src={benchmark}.wasabi.js></script> \
            <script async src=\"analysis.js\"></script>       \
        """

        html_content = open(source_html_path, 'r').read()
        html_content_updated = re.sub(pattern, replacement, html_content)
        open(destin_html_path, 'w').write(html_content_updated)
