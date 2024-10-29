# -*- coding: utf-8 -*-
from config import working_directory, minimum_major_node_version, minimum_major_wasm_merge_version
import os
import subprocess
import re

def setup_workspace():
    # Setup directories
    os.makedirs(working_directory, exist_ok=True)

    # Check minimum node version
    node_version = subprocess.run(['bash', '-c', 'node --version'], capture_output=True, text=True)
    node_version = node_version.stdout.strip()
    node_version_pattern = r'v(\d+)\.(\d+)\.(\d+)'
    node_version_pattern_match = re.match(node_version_pattern, node_version)
    assert node_version_pattern_match, f'`node --version` yielded (stdout) `{node_version}` which did not match an expected version output'
    [major, minor, patch] = [node_version_pattern_match.group(i) for i in [1,2,3]]
    minimum_major = minimum_major_node_version
    assert int(major) >= minimum_major, f'Version of Node {node_version} too low; found {major} but requires {minimum_major}'

    # Check minimum wasm-merge version
    wasm_merge_version = subprocess.run(['bash', '-c', 'wasm-merge --version'], capture_output=True, text=True)
    wasm_merge_version = wasm_merge_version.stdout.strip()
    wasm_merge_version_pattern = r'wasm-merge version (\d+)'
    wasm_merge_version_pattern_match = re.match(wasm_merge_version_pattern, wasm_merge_version)
    assert wasm_merge_version_pattern_match, f'`wasm-merge --version` (stdout) yielded `{wasm_merge_version}` which did not match an expected version output'
    [major] = [wasm_merge_version_pattern_match.group(i) for i in [1]]
    minimum_major = minimum_major_wasm_merge_version
    assert int(major) >= minimum_major, f'Version of wasm-merge {wasm_merge_version} too low; found {major} but requires {minimum_major}'
