#!/usr/bin/env bash
set -e  # stop at first error

[[ -z "$1" ]] && { echo "Please give package reference name as first parameter" ; exit 1; }
REF_NAME=$1

mkdir -p OpenCombat_${REF_NAME}_Linux
cp target/release/battle_gui OpenCombat_${REF_NAME}_Linux
cp target/release/battle_server OpenCombat_${REF_NAME}_Linux
cp -r resources OpenCombat_${REF_NAME}_Linux
cp assets/linux/* OpenCombat_${REF_NAME}_Linux
mkdir -p OpenCombat_${REF_NAME}_Linux/assets
cp assets/*deployment* OpenCombat_${REF_NAME}_Linux/assets/
cp LICENSE OpenCombat_${REF_NAME}_Linux
cp CREDITS OpenCombat_${REF_NAME}_Linux
zip -r OpenCombat_${REF_NAME}_Linux.zip OpenCombat_${REF_NAME}_Linux
