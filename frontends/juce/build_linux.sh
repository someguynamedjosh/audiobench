#!/bin/bash

# Exit on any errors
set -e

echo "This script will:"
echo "1. Build ProJUCEr"
echo "2. Build AudioBench"
echo "3. Build the JUCE frontend for AudioBench"
echo ""

BUILD_DIR="juce_lib/extras/Projucer/Builds/LinuxMakefile"
if [ -f "juce_lib/success_marker.txt" ]; then
    echo "Projucer already built"
else
    echo "Cloning JUCE..."
    rm -rf juce_lib
    git clone "https://github.com/juce-framework/JUCE" juce_lib

    echo "Setting GPL mode..."
    CONFIG_PATH="juce_lib/extras/Projucer/JuceLibraryCode/AppConfig.h"
    sed -i "s/JUCER_ENABLE_GPL_MODE 0/JUCER_ENABLE_GPL_MODE 1/g" "$CONFIG_PATH"

    echo "Building project..."
    ROOT_DIR=$(pwd)
    cd $BUILD_DIR
    make

    echo "Projucer built successfully!"
    cd $ROOT_DIR
    echo "success_marker" > juce_lib/success_marker.txt
fi

echo ""
echo "Building AudioBench..."
cd ../../
cargo build
cd frontends/juce
echo "Success!"

echo ""
echo "Building JUCE frontend..."
PROJUCER_PATH="$BUILD_DIR/build/Projucer"
$PROJUCER_PATH --resave AudioBench.jucer
echo "Success!"
