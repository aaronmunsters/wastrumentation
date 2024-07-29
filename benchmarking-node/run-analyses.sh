#!/usr/bin/env bash

(
    ECHO "##########################################"
    ECHO "RUNNING BENCHMARKS IN <add_two_i32_wasabi>"
    ECHO "##########################################"

    cd add_two_i32_wasabi
    rm -rf working-directory
    bash script.sh
    rm -rf working-directory
)

(
    ECHO "###################################################"
    ECHO "RUNNING BENCHMARKS IN <add_two_i32_wastrumentation>"
    ECHO "###################################################"

    cd add_two_i32_wastrumentation
    rm -rf working-directory
    bash script.sh
    rm -rf working-directory
)

(
    ECHO "##################################"
    ECHO "RUNNING BENCHMARKS IN <boa_wasabi>"
    ECHO "##################################"

    cd boa_wasabi
    rm -rf working-directory
    bash script.sh
    rm -rf working-directory
)

(
    ECHO "###########################################"
    ECHO "RUNNING BENCHMARKS IN <boa_wastrumentation>"
    ECHO "###########################################"

    cd boa_wastrumentation
    rm -rf working-directory
    bash script.sh
    rm -rf working-directory
)
