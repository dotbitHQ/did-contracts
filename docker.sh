#!/usr/bin/env bash

# Docker image name
DOCKER_IMAGE="thewawar/ckb-capsule:2021-08-16"
# Docker container name
DOCKER_CONTAINER="capsule-dev-"`whoami`
# Name of capsule cache volume
CACHE_VOLUME="capsule-cache"

function is_feature_available() {
    case $1 in
    dev | local | testnet2 | testnet3 | mainnet) ;;

    *)
        echo "Feature $1 is invalid, please choose one of dev|local|testnet2|testnet3|mainnet ."
        exit 1
        ;;
    esac
}

# Support `--release --dev/local/testnet/mainnet` or just `--dev/local/testnet/mainnet`
is_release=false
feature="mainnet"
function parse_args() {
    if [[ $1 == "--release" ]]; then
        is_release=true

        if [[ ! -z $2 ]]; then
            tmp=$2
            feature=${tmp:2}

            is_feature_available $feature
            if [[ $feature == "testnet3" || $feature == "testnet2" ]]; then
              feature="testnet"
            fi
        fi
    else
        if [[ ! -z $1 ]]; then
            tmp=$1
            feature=${tmp:2}

            is_feature_available $feature
            if [[ $feature == "testnet3" || $feature == "testnet2" ]]; then
              feature="testnet"
            fi
        fi
    fi
}

function create_output_dir() {
    if [[ $is_release == true ]]; then
        if [[ ! -d ./build/release ]]; then
            mkdir -p ./build/release
        fi
    else
        if [[ ! -d ./build/debug ]]; then
            mkdir -p ./build/debug
        fi
    fi
}

function build() {
    local contract=$1

    #    echo "is_release="$is_release "feature="$feature
    #    exit 0

    if [[ ! -d contracts/$contract ]]; then
        echo "Contract ${contract} is not exists, please check for spelling errors."
        exit 1
    fi

    if [[ $is_release == true ]]; then
        command="RUSTFLAGS=\"-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments\" cargo build --release --features \"${feature}\" --target riscv64imac-unknown-none-elf && ckb-binary-patcher -i /code/target/riscv64imac-unknown-none-elf/release/${contract} -o /code/target/riscv64imac-unknown-none-elf/release/${contract}"
        echo "Run build command: "$command

        # Build release version
        docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}"  &&\
        docker exec -it -w /code $DOCKER_CONTAINER bash -c \
            "cp /code/target/riscv64imac-unknown-none-elf/release/${contract} /code/build/release/"
    else
        command="cargo build --features \"${feature}\" --target riscv64imac-unknown-none-elf && ckb-binary-patcher -i /code/target/riscv64imac-unknown-none-elf/debug/${contract} -o /code/target/riscv64imac-unknown-none-elf/debug/${contract}"
        echo "Run build command: "$command

        # Build debug version
        docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}" &&\
        docker exec -it -w /code $DOCKER_CONTAINER bash -c \
            "cp /code/target/riscv64imac-unknown-none-elf/debug/${contract} /code/build/debug/"
    fi
    ret=$?

    if [[ $ret -ne 0 ]]; then
        echo "Build contract failed, exit code ($ret)."
        exit $ret
    else
        echo "Build contract succeeded."
    fi
}

function build_all() {
    local dirs=$(ls -a contracts)
    for contract in $dirs; do
        if [[ $contract != "." && $contract != ".." && -d contracts/$contract ]]; then
            build $contract $1
        fi
    done
}

case $1 in
start)
    dir="$(dirname $PWD)"
    if [[ $2 == "-b" || $2 == "--background" ]]; then
        docker run -d -t --rm \
            --name $DOCKER_CONTAINER \
            -v ${dir}/das-contracts:/code \
            -v ${dir}/das-types:/das-types \
            -v $CACHE_VOLUME:/root/.cargo \
            -e RUSTFLAGS="-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments" \
            -e CAPSULE_TEST_ENV=debug \
            $DOCKER_IMAGE bin/bash &>/dev/null
    else
        docker run -it --rm \
            --name $DOCKER_CONTAINER \
            -v ${dir}/das-contracts:/code \
            -v ${dir}/das-types:/das-types \
            -v $CACHE_VOLUME:/root/.cargo \
            -e RUSTFLAGS="-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments" \
            -e CAPSULE_TEST_ENV=debug \
            $DOCKER_IMAGE bin/bash
    fi
    ;;
stop)
    uuid=$(docker ps -a | grep ${DOCKER_IMAGE} | awk '{print $1}')
    if [[ ${uuid} != "" ]]; then
        docker stop ${uuid}
    fi
    ;;
build)
    parse_args $3 $4
    echo "Arguments: \$contract="$2 "\$is_release="$is_release "\$feature="$feature

    create_output_dir
    build $2
    ;;
build-all)
    parse_args $2 $3
    echo "Arguments: \$is_release="$is_release "\$feature="$feature

    create_output_dir
    build_all
    ;;
test)
    docker exec -it -w /code $DOCKER_CONTAINER bash -c "cargo test -p tests -- --nocapture"
    ;;
*)
    echo "Unsupported capsule command."
    exit 0
    ;;
esac

if [ $? -ne 0 ]; then
    echo "Build contracts failed. ❌"
    exit $?
else
    echo "Done ✔"
fi
