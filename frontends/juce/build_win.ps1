$ErrorActionPreference = "Stop"

echo "This script will:"
echo "1. Build Audiobench"
echo "2. Build the JUCE frontend for Audiobench"
echo "This version of the script only makes a release version because I don't"
echo "know how to use Powershell and I don't plan on learning."

$Env:CARGO_CFG_TARGET_FEATURE="crt-static"
cargo build --release
mkdir artifacts -ea 0
mkdir _build -ea 0

$Env:PROJECT_ROOT = Resolve-Path "../.."
# C compiler expects forward slashes.
$Env:PROJECT_ROOT = $Env:PROJECT_ROOT -replace "\\", "/"
$Env:RUST_OUTPUT_DIR = "$Env:PROJECT_ROOT/target/release"
cd _build
cmake -G"Visual Studio 16 2019" -A x64 -Thost=x64 ..
cd ..
cmake --build _build --config Release

Tree _build/Audiobench_artefacts/ /F
cp _build/Audiobench_artefacts/Release/Standalone/Audiobench.exe artifacts/Audiobench_Windows_x64_Standalone.exe
cp _build/Audiobench_artefacts/Release/VST3/Audiobench.vst3/Contents/x86_64-win/Audiobench.vst3 artifacts/Audiobench_Windows_x64_VST3.vst3
