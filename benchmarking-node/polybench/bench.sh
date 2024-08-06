#!/usr/bin/env bash

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

mkdir -p ${workingdir}
cd ${workingdir}

function download-unarchive () {
    local archive_path="$1"
    local archive_file="$2"
    local archive_link="$3"
    if [[ -d ${archive_path} ]]; then
        echo "${archive_path} found, using this as reference"
    else
        echo "${archive_path} not found, looking for archive file"
        if [[ ! -f ${archive_file} ]]; then
            echo "${archive_file} not found, looking to download it"
            wget                            \
                -O ${archive_file}      \
                --no-clobber                \
                --no-verbose                \
                --quiet                     \
                ${archive_link}
        fi
        if [[ ! -f ${archive_file} ]]; then
            echo "Could not download the ${archive_file} from ${archive_link}, aborting..."
            exit 0
        fi

        echo "Extracting ${archive_file} to ${archive_path}"
        mkdir -p ${archive_path}
        tar --extract --file ${archive_file} --directory ${archive_path}
    fi
}

### Fetch polybench benchmark suite
download-unarchive ${polybenchpath} ${polybencharchive} ${polybenchlink}

#################################################
### FETCH WEB BROWSER (locally, not globally) ###
### --> Very MacOS based ...                  ###
#################################################

# Line below is more Unix-based
# download-unarchive ${firefoxpath} ${firefoxarchive} ${firefoxlink_linux}

if [[ -d "`realpath /Volumes/Firefox*`" ]]; then
    echo "Assuming Firefox it already mounted, under \"`realpath /Volumes/Firefox*`\""
else
    echo "(temporary MacOS Firefox installation) Fetching"
    wget                    \
        -O firefox.dmg  \
        --no-clobber        \
        --no-verbose        \
        --quiet             \
        ${firefoxlink_macos}

    echo "(temporary MacOS Firefox installation) Mounting"
    hdiutil attach firefox.dmg -quiet
fi

firefox_binary=`realpath /Volumes/Firefox*/Firefox*.app/Contents/MacOS/firefox`

if [[ "${firefox_binary} --version" == "" ]]; then
    echo "Is the following a binary to a web browser?"
    echo ${firefox_binary}
    echo "Because I could not tell. Aborting."
fi
firefox_version=`"${firefox_binary}" --version`

###############################
### COMPILE BENCHMARK SUITE ###
###############################

cd ${polybenchpath}/${polybenchidentifier}
mkdir -p build/
while read sourcefile
do
	sourcedir=$(dirname $sourcefile)
	name=$(basename $sourcefile .c)

    # different dataset size per program to get similar runtimes
	dataset_size=$(sed -n -e "s/$name;//p" $dataset_size_list_path)

    # For documentation of Polybench/C, see README of downloaded ${polybencharchive}
    if [[ -f build/${name}.wasm && -f build/${name}.js && -f build/${name}.html ]]; then
        echo "[already compiled] skipping compilation $name ; ${dataset_size}"
    else
	    echo "Compiling $name (for ${dataset_size})"
        emcc                            \
            -O3                         \
            -I utilities                \
            -I $sourcedir               \
            utilities/polybench.c       \
            $sourcefile                 \
            -s ALLOW_MEMORY_GROWTH=1    \
            --emrun                     \
            -DPOLYBENCH_TIME            \
            -D$dataset_size             \
            -o build/$name.html
    fi

done < utilities/benchmark_list # <-- This file will dictate the input source files

##################
### BENCHMARKS ###
##################

# remove old firefox-profile if that existed
rm -rf firefox-profile && mkdir -p firefox-profile
firefox_args="--headless -no-remote -profile $(readlink -f firefox-profile)"

# create (new!) results file
results_file="runtime-analysis.csv"
echo > ${results_file} # create / clear ${results_file}
results_file_path=`readlink -f ${results_file}`

timeout="40s"
EXIT_STATUS_TIMEOUT=124

trap exit SIGINT SIGTERM # allow to break out of loop on Ctrl+C

for file in build/*.html; do
    # append findings to results file
	echo -n "\"${firefox_version}\", `basename ${file} .html`, " >> $results_file_path
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

cp ${results_file_path} ../../../${results_file}
