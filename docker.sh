#!/usr/bin/env bash

# Docker image name
DOCKER_IMAGE="thewawar/ckb-capsule:2021-08-16"
# Docker container name
DOCKER_CONTAINER="capsule-dev-"$(whoami)
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
    docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}" &&
      docker exec -it -w /code $DOCKER_CONTAINER bash -c \
        "cp /code/target/riscv64imac-unknown-none-elf/release/${contract} /code/build/release/"
  else
    command="RUSTFLAGS=\"-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments\" cargo build --features \"${feature}\" --target riscv64imac-unknown-none-elf && ckb-binary-patcher -i /code/target/riscv64imac-unknown-none-elf/debug/${contract} -o /code/target/riscv64imac-unknown-none-elf/debug/${contract}"
    echo "Run build command: "$command

    # Build debug version
    docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}" &&
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
    if [[ $contract != "." && $contract != ".." && $contract != "test-env" && $contract != "playground" && -d contracts/$contract ]]; then
      build $contract $1
    fi
  done
}

function switch_target_dir() {
  local expected=$1

  # If none cache directory found, rename the target directory as the first cache directory.
  if [[ ! -d target_test && ! -d target_build ]]; then
    if [[ $expected == "target_test" ]]; then
      mv target target_build
    else
      mv target target_test
    fi
  fi

  # If the expected cache directory exist, recover it as target directory.
  if [[ -d $expected ]]; then
    echo "Switching ${expected} to ./target ..."
    if [[ $expected == "target_test" ]]; then
      mv target target_build
    else
      mv target target_test
    fi
    mv $expected target
  fi
}

function join_by {
  local d=${1-} f=${2-}
  if shift 2; then printf %s "$f" "${@/#/$d}"; fi
}

case $1 in
start)
  dir="$(dirname $PWD)"
  if [[ $2 == "-b" || $2 == "--background" ]]; then
    docker run -d -t --rm \
      --name $DOCKER_CONTAINER \
      --network host \
      -v ${dir}/das-contracts:/code \
      -v ${dir}/das-types:/das-types \
      -v ${dir}/das-types-std:/das-types-std \
      -v $CACHE_VOLUME:/root/.cargo \
      -e RUSTFLAGS="-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments" \
      -e CAPSULE_TEST_ENV=debug \
      $DOCKER_IMAGE bin/bash &>/dev/null
  else
    docker run -it --rm \
      --name $DOCKER_CONTAINER \
      --network host \
      -v ${dir}/das-contracts:/code \
      -v ${dir}/das-types:/das-types \
      -v ${dir}/das-types-std:/das-types-std \
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

  switch_target_dir target_build
  create_output_dir
  build $2
  ;;
build-all)
  parse_args $2 $3
  echo "Arguments: \$is_release="$is_release "\$feature="$feature

  switch_target_dir target_build
  create_output_dir
  build_all
  ;;
test-debug)
  switch_target_dir target_test
  echo "Run test with name: $2"
  docker exec -it -w /code $DOCKER_CONTAINER bash -c "cargo test -p tests $2 -- --nocapture"
  ;;
test)
  switch_target_dir target_test
  echo "Run test with name: $2"
  docker exec -it -w /code $DOCKER_CONTAINER bash -c "cargo test -p tests $2"
  ;;
test-release)
  switch_target_dir target_test
  echo "Run test with name: $2"
  docker exec -it -w /code -e BINARY_VERSION=release $DOCKER_CONTAINER bash -c "cargo test -p tests $2"
  ;;
*)
  echo "Unsupported docker.sh command."
  exit 0
  ;;
esac

if [ $? -ne 0 ]; then
  echo "Build contracts failed. ❌"
  exit $?
else
  echo "Done ✔"
fi
