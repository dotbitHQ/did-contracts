#!/bin/bash

# Code path in container
CODE_PATH="/code"
# Docker image name
DOCKER_IMAGE="jjy0/ckb-capsule-recipe-rust:2020-9-28"
# Docker container name
DOCKER_CONTAINER="capsule-dev"
# Name of capsule cache volume
CACHE_VOLUME="capsule-cache"

function build() {
    local contract=$1

    if [ ! -d contracts/$contract ]; then
        echo "Contract ${contract} is not exists, please check for spelling errors."
        exit 1
    fi

    docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c \
        "cargo build --target riscv64imac-unknown-none-elf && ckb-binary-patcher -i /code/target/riscv64imac-unknown-none-elf/debug/${contract} -o /code/target/riscv64imac-unknown-none-elf/debug/${contract}"
    docker exec -it -w /code $DOCKER_CONTAINER bash -c \
        "cp /code/target/riscv64imac-unknown-none-elf/debug/${contract} /code/build/debug"
}

function build_all() {
    dirs=$(ls -a contracts)
    for contract in $dirs; do
        if [[ $contract != "." && $contract != ".." && -d contracts/$contract ]]; then
            build $contract
        fi
    done
}

case $1 in
start)
    docker run -it --rm \
        --name $DOCKER_CONTAINER \
        -v ${PWD}:/code \
        -v /Users/xieaolin/Documents/blockabc/das-types:/das-types \
        -v $CACHE_VOLUME:/root/.cargo \
        -e RUSTFLAGS="-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments" \
        -e CAPSULE_TEST_ENV=debug \
        $DOCKER_IMAGE bin/bash
    ;;
build)
    if [ -z $2 ];then
        build_all
    else
        build $2
    fi
    ;;
test)
    docker exec -it -w /code $DOCKER_CONTAINER bash -c "cargo test -p tests -- --nocapture"
    ;;
*)
    echo "Unsupported capsule command."
    exit 0
    ;;
esac

echo "Done ✔"
