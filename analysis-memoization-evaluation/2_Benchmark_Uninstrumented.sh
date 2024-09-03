#!/usr/bin/env bash

# Check if node version meets requirement
minimum_node_major_version=22
# assuming "node --version" output is of layout "vXX.XX.XX"
system_node_major_version=$(node --version | cut -d v -f 2 | cut -d "." -f 1)
if [[ $system_node_major_version -lt ${minimum_node_major_version} ]]; then
  echo "Node.js version must be ${minimum_node_major_version} or above. Please update Node.js and try again."
  exit 1
fi

cp call_benchmark.js \
   working-directory/

cd working-directory
node --experimental-wasm-multi-memory call_benchmark.js .
