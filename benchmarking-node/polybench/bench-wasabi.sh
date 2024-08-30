#!/usr/bin/env bash

benchmark_runs=10

workingdir="working-dir"
polybenchpath="polybench-c"
polybenchidentifier="polybench-c-4.2.1-beta"
polybencharchive="${polybenchidentifier}.tar.gz"
polybenchlink="https://downloads.sourceforge.net/project/polybench/${polybencharchive}"
dataset_size_list_path=$(readlink -f dataset_sizes)

firefoxlink_macos="https://download-origin.cdn.mozilla.net/pub/firefox/releases/129.0/mac/en-US/Firefox%20129.0.dmg"
firefoxlink_macos="https://download-origin.cdn.mozilla.net/pub/firefox/nightly/2024/08/2024-08-05-21-59-35-mozilla-central/firefox-131.0a1.en-US.mac.dmg"
firefoxlink_linux="https://download-origin.cdn.mozilla.net/pub/firefox/nightly/2024/08/2024-08-05-21-59-35-mozilla-central/firefox-131.0a1.en-US.linux-aarch64.tar.bz2"
firefoxarchive="firefox.tar.bz2"
firefoxpath="firefox"

function abort() {
    reason=$1
    echo "${reason}"
    echo "Was 'bench.sh' ran before? Aborting due to suspission this is not the case..."
    exit 0
}

if [[ ! -d ${workingdir} ]]; then abort "The folder ${workingdir} is not created yet."; fi
cd ${workingdir}

#################################################
### FETCH WEB BROWSER (locally, not globally) ###
### --> Very MacOS based ...                  ###
#################################################

# Line below is more Unix-based
# download-unarchive ${firefoxpath} ${firefoxarchive} ${firefoxlink_linux}

if [[ ! -d "`realpath /Volumes/Firefox*`" ]]; then abort "A benchmark instance of Firefox is not mounted?"; fi
firefox_binary=`realpath /Volumes/Firefox*/Firefox*.app/Contents/MacOS/firefox`

if [[ "${firefox_binary} --version" == "" ]]; then abort "Is ${firefox_binary} a binary to a web browser? Because I could not tell."; fi
firefox_version=`"${firefox_binary}" --version`

###############################
### COMPILE BENCHMARK SUITE ###
###############################

benchmark_list_path="${polybenchpath}/${polybenchidentifier}/utilities/benchmark_list"
if [[ ! -f ${benchmark_list_path} ]]; then abort "Could not find ${benchmark_list_path}"; fi

cd ${polybenchpath}/${polybenchidentifier}
mkdir -p build/
mkdir -p build-instrumented-wastrumentation/
while read sourcefile
do
	sourcedir=$(dirname $sourcefile)
	name=$(basename $sourcefile .c)

    # different dataset size per program to get similar runtimes
	dataset_size=$(sed -n -e "s/$name;//p" $dataset_size_list_path)

    # For documentation of Polybench/C, see README of downloaded ${polybencharchive}
    if [[ ! (-f build/${name}.wasm && -f build/${name}.js && -f build/${name}.html) ]]; then
        abort "Compilation for $name (for ${dataset_size}) did not happen..."
    fi

    cp build/${name}.js   build-instrumented-wastrumentation/${name}.js
    cp build/${name}.html build-instrumented-wastrumentation/${name}.html
    rm -f                 build-instrumented-wastrumentation/${name}.wasm

    echo "Instrumenting `realpath ./build/${name}.wasm`"
    cargo run -- \
        --input-program-path `realpath ./build/${name}.wasm`                                                        \
        --rust-analysis-toml-path `realpath ../../../../input-analyses/rust/call-stack-wastrumentation/Cargo.toml`  \
        --hooks CallBefore                                                                                          \
                CallAfter                                                                                           \
                CallIndirectBefore                                                                                  \
                CallIndirectAfter                                                                                   \
                GenericApply                                                                                        \
        --output-path "./build-instrumented-wastrumentation/${name}.wasm"

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

timeout="40s"
EXIT_STATUS_TIMEOUT=124

trap exit SIGINT SIGTERM # allow to break out of loop on Ctrl+C

total_input_programs=`ls -1 build-instrumented-wastrumentation/*.html | wc -l`
total_runs=$((${benchmark_runs}*${total_input_programs}))
iteration=0

for benchmark_run in `seq ${benchmark_runs}`; do
for file in build-instrumented-wastrumentation/*.html; do
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
