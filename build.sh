#!/bin/sh

features=" --features"
system=""
options=""
while getopt -g:--os:--release flag
do
  case ${flag} in
    g) features="${features} ${OPTARG}";;
    os) system="--target=${OPTARG}";;
    release) options="${options} --release"
  esac

done


if ["${features}" != " --features"]; then
  options="${options}${features}"
else
  options="${options} --features opengl" #default option!!!
fi
options="${options} ${system}"

echo "cargo build ${options}"

cargo build ${options}
