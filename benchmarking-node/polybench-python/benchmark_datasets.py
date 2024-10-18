# -*- coding: utf-8 -*-
import re
from typing import Dict

from config import dataset_size_list_name

class BenchmarkDatasetSizes:
    """
    Class that reads the `dataset_size` file upon
    initialization and turns it into:
    `benchmark_dataset_sizes: Dict[str, str]`
    """
    def __init__(self) -> None:
        # Open file
        dataset_size_list = open(dataset_size_list_name)
        pattern = r'^([\w-]+);(\w+_DATASET)$'

        # Initialize empty map
        self.benchmark_dataset_sizes: Dict[str, str] = {}

        # For each line in the file, match the pattern
        for line in dataset_size_list:
            matches = re.match(pattern, line)
            if not matches: continue

            # for each match inject it in the mapping
            program, size = matches.groups()
            self.benchmark_dataset_sizes[program] = size
