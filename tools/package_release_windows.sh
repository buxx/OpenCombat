#!/usr/bin/env bash
set -e  # stop at first error

[[ -z "$1" ]] && { echo "Please give package reference name as first parameter" ; exit 1; }
REF_NAME=$1

mkdir OpenCombat_${REF_NAME}_Win64
cp target/release/battle_gui.exe OpenCombat_${REF_NAME}_Win64
cp target/release/battle_server.exe OpenCombat_${REF_NAME}_Win64
cp -r resources OpenCombat_${REF_NAME}_Win64
cp assets/w64/* OpenCombat_${REF_NAME}_Win64
mkdir OpenCombat_${REF_NAME}_Win64/assets
cp assets/*deployment* OpenCombat_${REF_NAME}_Win64/assets/
cp LICENSE OpenCombat_${REF_NAME}_Win64
cp CREDITS OpenCombat_${REF_NAME}_Win64
find OpenCombat_${REF_NAME}_Linux -name "*.xcf" -type f -delete
zip -r OpenCombat_${REF_NAME}_Win64.zip OpenCombat_${REF_NAME}_Win64
