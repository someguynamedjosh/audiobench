#!/bin/bash

# Exit on any errors
set -e

echo "This script will:"
echo "1. Build ProJUCEr"
echo "2. Build AudioBench"
echo "3. Build the JUCE frontend for AudioBench"
echo "Run with argument 'projucer' to open the project in ProJUCEr."
echo "Run with argument 'run' to execute the standalone version for testing."
echo ""

ROOT_DIR=$(pwd)
PROJUCER_BUILD_DIR="juce_lib/extras/Projucer/Builds/LinuxMakefile"
PROJUCER_PATH="$PROJUCER_BUILD_DIR/build/Projucer"
SEPERATOR="========================================"
echo $SEPERATOR
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
    cd $PROJUCER_BUILD_DIR
    make

    echo "Projucer built successfully!"
    cd $ROOT_DIR
    echo "success_marker" > juce_lib/success_marker.txt
fi

if [ "$1" == "projucer" ]; then
    $PROJUCER_PATH AudioBench.jucer
    exit 0
fi

echo ""
echo $SEPERATOR
echo "Building AudioBench..."
cd ../../
cargo build
cd frontends/juce
echo "Success!"

echo ""
echo $SEPERATOR
echo "Building JUCE frontend..."
$PROJUCER_PATH --resave AudioBench.jucer
VST_BUILD_DIR="Builds/LinuxMakefile"
cd $VST_BUILD_DIR
# The build system cannot detect when the AudioBench library has changed, so we
# delete the build artifacts to force it to re-link them.
rm build/AudioBench*
make
cd $ROOT_DIR
echo "Success!"

if [ "$1" == "run" ]; then
    echo ""
    echo "Starting standalone version..."
    cd "$VST_BUILD_DIR/build"
    # Standalone version
    ./AudioBench
fi
