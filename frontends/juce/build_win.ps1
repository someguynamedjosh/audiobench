$ErrorActionPreference = "Stop"

echo "This script will:"
echo "1. Remove JUCE splash screen (Audiobench is GPLv3)"
echo "2. Build JUCE 6"
echo "3. Build Audiobench"
echo "4. Build the JUCE frontend for Audiobench"
echo "This version of the script only makes a release version because I don't"
echo "know how to use Powershell and I don't plan on learning."

echo "Removing JUCE splash..."
python remove_splash.py

mkdir juce6_built -ea 0
$Env:JUCE6_PREFIX = Resolve-Path "juce6_built"
$Env:JUCE6_PREFIX = $Env:JUCE6_PREFIX -replace "\\", "/"
cd juce_git
cmake -Bcmake-build-install -DCMAKE_INSTALL_PREFIX="$Env:JUCE6_PREFIX" -G"Visual Studio 16 2019" -A x64 -Thost=x64
cmake --build cmake-build-install --target install
cd ..
$Env:JUCE_DIR = "$Env:JUCE6_PREFIX/lib/cmake/JUCE-6.0.0"

cargo build --release
mkdir artifacts -ea 0
mkdir _build -ea 0

$Env:PROJECT_ROOT = Resolve-Path "../.."
# C compiler expects forward slashes.
$Env:PROJECT_ROOT = $Env:PROJECT_ROOT -replace "\\", "/"
$Env:RUST_OUTPUT_DIR = "$Env:PROJECT_ROOT/target/release"
cd _build
cmake -DJUCE_DIR="$Env:JUCE_DIR" -G"Visual Studio 16 2019" -A x64 -Thost=x64 ..
cd ..
cmake --build _build --config Release

Tree _build/Audiobench_artefacts/ /F
cp _build/Audiobench_artefacts/Release/Standalone/Audiobench.exe artifacts/Audiobench_Windows_x64_Standalone.exe
cp _build/Audiobench_artefacts/Release/VST3/Audiobench.vst3/Contents/x86_64-win/Audiobench.vst3 artifacts/Audiobench_Windows_x64_VST3.vst3
