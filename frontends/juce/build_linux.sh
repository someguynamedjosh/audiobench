#!/bin/bash

# Exit on any errors
set -e

echo "This script will:"
echo "1. Build AudioBench"
echo "2. Build the JUCE frontend for AudioBench"
echo "Run with argument 'run' to execute the standalone version for testing."
echo "Run with argument 'clean' to delete old build files before building."
echo ""

FRONTEND_ROOT=$(pwd)
export PROJECT_ROOT="$(readlink -f "$FRONTEND_ROOT/../..")"
export RUST_OUTPUT_DIR="$PROJECT_ROOT/target/debug"
SEPERATOR="========================================"

echo ""
echo $SEPERATOR
echo "Building AudioBench..."
cd ../../
if [ "$1" == "clean" ]; then
    cargo clean
fi
cargo build
cd frontends/juce
echo "Success!"

echo ""
echo $SEPERATOR
echo "Building JUCE frontend..."
if [ "$1" == "clean" ]; then
    rm -rf _build
fi
mkdir -p _build
cd _build
cmake ..
cd ..
cmake --build _build --config Debug
cp _build/AudioBench_artefacts/VST3/AudioBench.vst3/Contents/x86_64-linux/AudioBench.so ~/.vst3/
# The build system cannot detect when the AudioBench library has changed, so we
# delete the build artifacts to force it to re-link them.
# rm -f build/AudioBench*
# make
cd $FRONTEND_ROOT
echo "Success!"

if [ "$1" == "run" ]; then
    echo ""
    echo "Starting standalone version..."
    # Standalone version
    ./_build/AudioBench_artefacts/Standalone/AudioBench
fi
