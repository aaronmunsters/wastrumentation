#!/usr/bin/env bash

benchmark_runs=1

workingdir="working-dir"
polybenchpath="polybench-c"
polybenchidentifier="polybench-c-4.2.1-beta"
dataset_size_list_path=$(readlink -f dataset_sizes)

function abort() {
    reason=$1
    echo "${reason}"
    echo "Was 'bench.sh' ran before? Aborting due to suspission this is not the case..."
    exit 0
}

#################################################################
### ENSURE WORKING DIR, INPUT SUITE & WEB BROWSER ARE PRESENT ###
#################################################################

if [[ ! -d ${workingdir} ]]; then abort "The folder ${workingdir} is not created yet."; fi
cd ${workingdir}

if [[ ! -d "`realpath /Volumes/Firefox*`" ]]; then abort "A benchmark instance of Firefox is not mounted?"; fi
firefox_binary=`realpath /Volumes/Firefox*/Firefox*.app/Contents/MacOS/firefox`

if [[ "${firefox_binary} --version" == "" ]]; then abort "Is ${firefox_binary} a binary to a web browser? Because I could not tell."; fi
firefox_version=`"${firefox_binary}" --version`

benchmark_list_path="${polybenchpath}/${polybenchidentifier}/utilities/benchmark_list"
if [[ ! -f ${benchmark_list_path} ]]; then abort "Could not find ${benchmark_list_path}"; fi

###############################
### COMPILE BENCHMARK SUITE ###
###############################

cd ${polybenchpath}/${polybenchidentifier}
mkdir -p build/
instrumented_build_dir="build-instrumented-wastrumentation"
mkdir -p ${instrumented_build_dir}/
while read sourcefile
do
	sourcedir=$(dirname $sourcefile)
	name=$(basename $sourcefile .c)

    # different dataset size per program to get similar runtimes
	dataset_size=$(sed -n -e "s/$name;//p" $dataset_size_list_path)

    # Skip if compilation did not happen for input program
    if [[ ! (-f build/${name}.wasm && -f build/${name}.js && -f build/${name}.html) ]]; then
        abort "Compilation for $name (for ${dataset_size}) did not happen..."
    fi

    # For documentation of Polybench/C, see README of downloaded ${polybencharchive}
    if [[ -f ${instrumented_build_dir}/${name}.wasm && -f ${instrumented_build_dir}/${name}.js && -f ${instrumented_build_dir}/${name}.html ]]; then
        echo "[already instrumented] skipping instrumentation $name ; ${dataset_size}"
        continue
    fi

    echo "[instrumenting] $name ; ${dataset_size}"
    cp build/${name}.js   ${instrumented_build_dir}/${name}.js
    cp build/${name}.html ${instrumented_build_dir}/${name}.html
    rm -f                 ${instrumented_build_dir}/${name}.wasm

    echo "Instrumenting `realpath ./build/${name}.wasm`"
    cargo run --                                                                                        \
        --input-program-path `realpath ./build/${name}.wasm`                                            \
        --rust-analysis-toml-path `realpath ../../../../input-analyses/rust/instruction-mix/Cargo.toml` \
        `# all hooks except call_generic`                                                               \
        --hooks call-pre call-post call-indirect-pre call-indirect-post if-then if-then-post            \
                if-then-else if-then-else-post branch branch-if branch-table select unary               \
                binary drop return const local global store load memory-size memory-grow                \
                block-pre block-post loop-pre loop-post                                                 \
        --output-path "./${instrumented_build_dir}/${name}.wasm"
done < utilities/benchmark_list # <-- This file will dictate the input source files

##################
### BENCHMARKS ###
##################

# remove old firefox-profile if that existed
rm -rf firefox-profile && mkdir -p firefox-profile
firefox_args="--headless -no-remote -profile $(readlink -f firefox-profile)"

# create (new!) results file
results_file="runtime-analysis-wastrumentation.csv"
echo > ${results_file} # create / clear ${results_file}
echo "runtime_environment,benchmark,performance" >> ${results_file}
results_file_path=`readlink -f ${results_file}`

timeout="4000s"
EXIT_STATUS_TIMEOUT=124

trap exit SIGINT SIGTERM # allow to break out of loop on Ctrl+C

total_input_programs=`ls -1 ${instrumented_build_dir}/*.html | wc -l`
total_runs=$((${benchmark_runs}*${total_input_programs}))
iteration=0

for benchmark_run in `seq ${benchmark_runs}`; do
for file in ${instrumented_build_dir}/*.html; do
    echo "[BENCHMARK PROGRESS]: ${iteration}/${total_runs}"; iteration=$((${iteration}+1))

    # append findings to results file
	echo -n "\"${firefox_version} (wastrumentation)\", `basename ${file} .html`, " >> $results_file_path
	timeout ${timeout} `# execute command of line below, wrapped in timeout shield of ${timeout} seconds` \
    emrun \
        --log_stdout "${results_file_path}" `# Write findings to file             ` \
        --browser "${firefox_binary}"       `# Rely on downloaded browser         ` \
        --browser_args "${firefox_args}"    `# Pass custom arguments to browser   ` \
        --kill_exit                         `# Kill browser process on exit       ` \
        "${file}"                           `# Target benchmark                   `

    # If benchmark ran out of time, report this too
    if [ $? -eq ${EXIT_STATUS_TIMEOUT} ]; then
        echo "timeout ${timeout}" >> $results_file_path
    fi
done
done

cp ${results_file_path} ../../../${results_file}
