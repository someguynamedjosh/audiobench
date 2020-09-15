#!/bin/bash

# Exit on any errors
set -e

echo "This script will:"
echo "1. Remove JUCE splash screen (Audiobench is GPLv3)"
echo "2. Build Audiobench"
echo "3. Build the JUCE frontend for Audiobench"
echo "Run with argument 'run' to execute the standalone version for testing."
echo "Run with argument 'perf' to profile the standalone version with perf."
echo "Run with argument 'clean' to delete old build files before building."
echo "Run with argument 'release' to build optimized release binaries."
echo ""


export PROJECT_ROOT=$(pwd)
if [ "$1" == "release" ]; then
    export RUST_OUTPUT_DIR="$PROJECT_ROOT/target/release"
else
    export RUST_OUTPUT_DIR="$PROJECT_ROOT/target/debug"
fi
SEPERATOR="========================================"

cd components/juce_frontend/
FRONTEND_ROOT=$(pwd)

echo "Removing JUCE splash..."
python remove_splash.py
echo ""
echo $SEPERATOR
echo "Building Audiobench..."
if [ "$1" == "clean" ]; then
    cargo clean
fi
if [ "$1" == "release" ]; then
    cargo build --release -p audiobench
else
    cargo build -p audiobench
fi
echo "Success!"

echo ""
echo $SEPERATOR
echo "Building JUCE frontend..."
if [ "$1" == "clean" ]; then
    rm -rf "$PROJECT_ROOT/artifacts"
    rm -rf _build
fi
mkdir -p "$PROJECT_ROOT/artifacts"
mkdir -p _build
cd _build
if [ "$1" == "release" ]; then
    cmake -Wno-dev -DCMAKE_BUILD_TYPE=Release ..
else
    cmake -Wno-dev -DCMAKE_BUILD_TYPE=Debug ..
fi
cd ..
if [ "$1" == "release" ]; then
    cmake --build _build --config Release 
    cp _build/Audiobench_artefacts/Release/Standalone/Audiobench "$PROJECT_ROOT/artifacts/Audiobench_Linux_x64_Standalone.bin"
    cp _build/Audiobench_artefacts/Release/VST3/Audiobench.vst3/Contents/x86_64-linux/Audiobench.so "$PROJECT_ROOT/artifacts/Audiobench_Linux_x64_VST3.so"
else
    cmake --build _build --config Debug
    cp _build/Audiobench_artefacts/Debug/Standalone/Audiobench "$PROJECT_ROOT/artifacts/Audiobench_Linux_x64_Standalone.bin"
    cp _build/Audiobench_artefacts/Debug/VST3/Audiobench.vst3/Contents/x86_64-linux/Audiobench.so "$PROJECT_ROOT/artifacts/Audiobench_Linux_x64_VST3.so"
fi
echo "Success!"

cd "$PROJECT_ROOT"
if [ "$1" == "run" ]; then
    echo ""
    echo "Starting standalone version..."
    # Standalone version
    ./artifacts/Audiobench_Linux_x64_Standalone.bin
fi
if [ "$1" == "perf" ]; then
    echo ""
    echo "Profiling standalone version..."
    perf record -g ./artifacts/Audiobench_Linux_x64_Standalone.bin
fi
