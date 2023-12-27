#!/usr/bin/env bash
set -e  # stop at first error

[[ -z "$1" ]] && { echo "Please give package reference name as first parameter" ; exit 1; }
FOLDER_NAME=OpenCombat_${1}_Linux

mkdir -p ${FOLDER_NAME}
cp target/release/battle_gui ${FOLDER_NAME}
cp target/release/battle_server ${FOLDER_NAME}
cp target/release/oc_launcher ${FOLDER_NAME}/start
cp -r resources ${FOLDER_NAME}
cp assets/linux/* ${FOLDER_NAME}
mkdir -p ${FOLDER_NAME}/assets
cp assets/*deployment* ${FOLDER_NAME}/assets/
cp LICENSE ${FOLDER_NAME}
cp CREDITS ${FOLDER_NAME}
find ${FOLDER_NAME} -name "*.xcf" -type f -delete
zip -r ${FOLDER_NAME}.zip ${FOLDER_NAME}
