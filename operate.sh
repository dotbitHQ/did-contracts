#!/bin/sh
PROJ_ROOT=`pwd`"/../"
CONTRACT_ROOT=${PROJ_ROOT}"/das-contracts/"
CONTRACT_RELEASE_DIR=${CONTRACT_ROOT}"/build/release/"
TYPES_ROOT=${PROJ_ROOT}"/das-types/"
DAS_TOOL=${PROJ_ROOT}"/das-tool/"
CELL_DATA_ROOT=${PROJ_ROOT}/"cell-data-generator/"

docker ps | grep "jjy0/ckb-capsule-recipe-rust" &> /dev/null
if [ $? -ne 0 ]; then
	echo "ckb-capsule-recipe-rust is not up, we will start up it"
	cd ${CONTRACT_ROOT};./docker.sh start --background
fi
contract_bin=$1
net_env=$4

cd ${CONTRACT_ROOT};./docker.sh build ${contract_bin} --release --${net_env}
if [ $? -eq 0 ]; then
	c_args=" -c "${CONTRACT_RELEASE_DIR}${contract_bin}
	w_args=" -w "${CELL_DATA_ROOT}${contract_bin}"-generator/"
	N_args=" -N "${contract_bin}
#echo ${c_args}
#echo ${w_args}

	rest_args=$*
	case $2 in
	deploy|update-contract)
		python3 ${DAS_TOOL}/main.py ${rest_args#* } ${c_args} ${N_args}
		;;
	onelong)
		python3 ${DAS_TOOL}/main.py ${rest_args#* } ${c_args} ${w_args} ${N_args}
		;;
	update-cell)
		python3 ${DAS_TOOL}/main.py ${rest_args#* } ${w_args}
		;;
	*)
		echo "check the command"
		;;
	esac
		
fi
