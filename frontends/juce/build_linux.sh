#!/bin/bash

# Exit on any errors
set -e

echo "This script will:"
echo "1. Build Audiobench"
echo "2. Build the JUCE frontend for Audiobench"
echo "Run with argument 'run' to execute the standalone version for testing."
echo "Run with argument 'clean' to delete old build files before building."
echo "Run with argument 'release' to build optimized release binaries."
echo ""

FRONTEND_ROOT=$(pwd)
export PROJECT_ROOT="$(readlink -f "$FRONTEND_ROOT/../..")"
if [ "$1" == "release" ]; then
    export RUST_OUTPUT_DIR="$PROJECT_ROOT/target/release"
else
    export RUST_OUTPUT_DIR="$PROJECT_ROOT/target/debug"
fi
SEPERATOR="========================================"

echo ""
echo $SEPERATOR
echo "Building Audiobench..."
cd ../../
if [ "$1" == "clean" ]; then
    cargo clean
fi
if [ "$1" == "release" ]; then
    cargo build --release
else
    cargo build
fi
cd frontends/juce
echo "Success!"

echo ""
echo $SEPERATOR
echo "Building JUCE frontend..."
if [ "$1" == "clean" ]; then
    rm -rf artifacts
    rm -rf _build
fi
mkdir -p artifacts
mkdir -p _build
cd _build
cmake ..
cd ..
if [ "$1" == "release" ]; then
    cmake --build _build --config Release
else
    cmake --build _build --config Debug
fi
cd $FRONTEND_ROOT
cp _build/Audiobench_artefacts/Standalone/Audiobench artifacts/Audiobench_Linux_x64_Standalone.bin
cp _build/Audiobench_artefacts/VST3/Audiobench.vst3/Contents/x86_64-linux/Audiobench.so artifacts/Audiobench_Linux_x64_VST3.so
echo "Success!"

if [ "$1" == "run" ]; then
    echo ""
    echo "Starting standalone version..."
    # Standalone version
    ./artifacts/Audiobench_Linux_x64_Standalone.bin
fi
