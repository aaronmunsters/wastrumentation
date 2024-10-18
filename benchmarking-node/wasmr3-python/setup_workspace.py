# -*- coding: utf-8 -*-
from config import working_directory
import os

def setup_workspace():
    # Setup directories
    os.makedirs(working_directory, exist_ok=True)
