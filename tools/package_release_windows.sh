#!/usr/bin/env bash
set -e  # stop at first error

[[ -z "$1" ]] && { echo "Please give package reference name as first parameter" ; exit 1; }
FOLDER_NAME=OpenCombat_${1}_Win64

mkdir ${FOLDER_NAME}
cp target/release/battle_gui.exe ${FOLDER_NAME}
cp target/release/battle_server.exe ${FOLDER_NAME}
cp -r resources ${FOLDER_NAME}
cp assets/w64/* ${FOLDER_NAME}
mkdir ${FOLDER_NAME}/assets
cp assets/*deployment* ${FOLDER_NAME}/assets/
cp LICENSE ${FOLDER_NAME}
cp CREDITS ${FOLDER_NAME}
find OpenCombat_${REF_NAME}_Linux -name "*.xcf" -type f -delete
zip -r ${FOLDER_NAME}.zip ${FOLDER_NAME}
