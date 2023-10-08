#!/bin/bash

SCHEMA_PATH="${PWD}/schemas"
DIST_RUST_PATH="${PWD}/rust"
DIST_GO_PATH="${PWD}/go"
DIST_JS_PATH="${PWD}/js"

function compile() {
    local language=$1
    local schema_path=$2
    local dist_path=$3
    local suffix=".txt"

    case $1 in
    rust) suffix=".rs" ;;
    go)   suffix=".go" ;;
    js)   suffix=".js" ;;
    esac

    if [[ ! -d $dist_path ]]; then
        mkdir -p $dist_path
    fi

    # walk through all directories recursively
    files=$(ls -a $schema_path)
    for file in $files; do
        if [[ $file != .* ]]; then
            if [ ! -d "${file}" ]; then
                echo "Compile ${schema_path}/${file} to ${dist_path}/${file%.*}${suffix}"
                case $language in
                js)
                    moleculec --language - --schema-file ${schema_path}/${file} --format json > ./tmp-schema.json
                    moleculec-es -inputFile ./tmp-schema.json -outputFile ${dist_path}/${file%.*}${suffix} -generateTypeScriptDefinition -hasBigInt
                    rm ./tmp-schema.json
                    ;;
                *)
                    moleculec --language $language --schema-file ${schema_path}/${file} >${dist_path}/${file%.*}${suffix}
                    ;;
                esac
            else
                compile $language $schema_path $dist_path
            fi
        fi
    done
}

case $1 in
rust)
    compile rust $SCHEMA_PATH $DIST_RUST_PATH/src/schemas
    cargo fmt
    ;;
go)
    compile go $SCHEMA_PATH $DIST_GO_PATH/src
    ;;
js)
    compile js $SCHEMA_PATH $DIST_JS_PATH/src
    ;;
*)
    echo "Unsupported compiling target."
    exit 0
    ;;
esac

echo "Done ✔"
