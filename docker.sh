#!/usr/bin/env bash

# Docker image name
DOCKER_IMAGE="dotbitteam/ckb-dev-all-in-one:0.0.1"
COMPILING_TARGET="riscv64imac-unknown-none-elf"
COMPILING_FLAGS="-Z pre-link-arg=-zseparate-code -Z pre-link-arg=-zseparate-loadable-segments"
COMPILING_RELEASE_FLAGS="-C link-arg=-s"
# Docker container name
DOCKER_CONTAINER="capsule-dev"${PWD//\//_}
# Name of capsule cache volume
CACHE_VOLUME="capsule-cache"

function is_feature_available() {
  case $1 in
  dev | local | testnet2 | testnet3 | mainnet) ;;

  *)
    switch_target_dir host

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
    switch_target_dir host

    echo "Contract ${contract} is not exists, please check for spelling errors."
    exit 1
  fi

  if [[ $is_release == true ]]; then echo "release: true"; else echo "release: false"; fi
  echo "feature: " $feature

  local profile="debug"
  local rust_flags=${COMPILING_FLAGS}
  local binary_path=""

  if [[ $is_release == true ]]; then
    rust_flags="${rust_flags} ${COMPILING_RELEASE_FLAGS}"
    command="RUSTFLAGS=\"${rust_flags}\" cargo build --release --features \"${feature}\" --target ${COMPILING_TARGET}"
    profile="release"
  else
    command="RUSTFLAGS=\"${rust_flags}\" cargo build --features \"${feature}\" --target ${COMPILING_TARGET}"
    echo "Run build command: "$command

    # Build debug version
    docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}" &&
      docker exec -it -w /code $DOCKER_CONTAINER bash -c \
        "cp /code/target/${COMPILING_TARGET}/debug/${contract} /code/build/debug/"
  fi

  if [[ -d "contracts/$contract/examples" ]]; then
    command="${command} --examples"
    binary_path="/code/target/${COMPILING_TARGET}/${profile}/examples/${contract}"
  else
    binary_path="/code/target/${COMPILING_TARGET}/${profile}/${contract}"
  fi

  command="${command} && ckb-binary-patcher -i ${binary_path} -o ${binary_path}"
  echo "Run build command: "$command

    # Build release version
  docker exec -it -w /code/contracts/$contract $DOCKER_CONTAINER bash -c "${command}" &&
    docker exec -it -w /code $DOCKER_CONTAINER bash -c \
      "cp ${binary_path} /code/build/${profile}/"

  ret=$?

  if [[ $ret -ne 0 ]]; then
    switch_target_dir host

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

  if [[ $expected == "docker" ]]; then
    if [[ -d target ]]; then
      mv target target_host
    fi
    if [[ -d target_docker ]]; then
      mv target_docker target
    fi
  else
    if [[ -d target ]]; then
      mv target target_docker
    fi
    if [[ -d target_host ]]; then
      mv target_host target
    fi
  fi
}

function join_by {
  local d=${1-} f=${2-}
  if shift 2; then printf %s "$f" "${@/#/$d}"; fi
}

case $1 in
start)
  dir=$PWD
  if [[ $2 == "-b" || $2 == "--background" ]]; then
    docker run -d -t --rm \
      --name $DOCKER_CONTAINER \
      --network host \
      -v ${dir}:/code \
      -v $CACHE_VOLUME:/root/.cargo \
      -v ~/.gitconfig:/root/.gitconfig:ro \
      $DOCKER_IMAGE /bin/bash &>/dev/null
  else
    docker run -it --rm \
      --name $DOCKER_CONTAINER \
      --network host \
      -v ${dir}:/code \
      -v ~/.gitconfig:/root/.gitconfig:ro \
      -v $CACHE_VOLUME:/root/.cargo \
      $DOCKER_IMAGE \
      /bin/bash
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

  switch_target_dir docker
  create_output_dir
  build $2
  switch_target_dir host
  ;;
build-all)
  parse_args $2 $3
  echo "Arguments: \$is_release="$is_release "\$feature="$feature

  switch_target_dir docker
  create_output_dir
  build_all
  switch_target_dir host
  ;;
test-debug)
  switch_target_dir docker
  echo "Run test with name: $2"
  docker exec -it -w /code -e BINARY_VERSION=debug $DOCKER_CONTAINER bash -c "cargo test -p tests $2 -- --nocapture"
  switch_target_dir host
  ;;
test)
  switch_target_dir docker
  echo "Run test with name: $2"
  docker exec -it -w /code -e BINARY_VERSION=debug $DOCKER_CONTAINER bash -c "cargo test -p tests $2"
  switch_target_dir host
  ;;
test-release)
  switch_target_dir docker
  echo "Run test with name: $2"
  docker exec -it -w /code -e BINARY_VERSION=release $DOCKER_CONTAINER bash -c "cargo test -p tests $2"
  switch_target_dir host
  ;;
perf-release)
  switch_target_dir docker
  echo "Run test with name: $2"
  docker exec -it -w /code -e BINARY_VERSION=release $DOCKER_CONTAINER bash -c "cargo test -p tests $2 -- --nocapture"
  switch_target_dir host
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
