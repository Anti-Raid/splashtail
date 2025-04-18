#!/bin/bash

# Usage function
usage() {
  echo "Usage: $0 <path to khronos folder> <path to docs folder> <path to lune>"
  exit 1
}

# Get path to khronos folder from user
KHRONOS_FOLDER=$1
if [ -z "$KHRONOS_FOLDER" ]; then
  usage 
fi

# Get path to docs folder from user
DOCS_FOLDER=$2
if [ -z "$DOCS_FOLDER" ]; then
  usage
fi

# Ensure DOCS_FOLDER is absolute
if [ "${DOCS_FOLDER:0:1}" != "/" ]; then
  DOCS_FOLDER=$(pwd)/$DOCS_FOLDER
fi

LUNE_PATH=$3
if [ -z "$LUNE_PATH" ]; then
  usage
fi

# Ensure LUNE_PATH is absolute
if [ "${LUNE_PATH:0:1}" != "/" ]; then
  LUNE_PATH=$(pwd)/$LUNE_PATH
fi

cd $KHRONOS_FOLDER
rm -rf $DOCS_FOLDER/src/dev/templating-api
mkdir $DOCS_FOLDER/src/dev/templating-api
$LUNE_PATH run createdocs.luau $DOCS_FOLDER/src/dev/templating-api