# -*- coding: utf-8 -*-
from config import working_directory, minimum_major_node_version
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
    [major, minor, patch] = [node_version_pattern_match.group(i) for i in [1,2,3]]
    minimum_major = minimum_major_node_version
    assert int(major) >= minimum_major, f'Version of Node {node_version} too low; found {major} but requires {minimum_major}'
