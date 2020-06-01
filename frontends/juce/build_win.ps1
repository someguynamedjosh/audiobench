$ErrorActionPreference = "Stop"

echo "This script will:"
echo "1. Build Audiobench"
echo "2. Build the JUCE frontend for Audiobench"
echo "This version of the script only makes a release version because I don't"
echo "know how to use Powershell and I don't plan on learning."

cargo build --release
md artifacts -ea 0
md _build -ea 0
cd _build
cmake ..
cd ..
cmake --build _build --config Release
Tree _build\Audiobench_artefacts\ /F
cp _build\Audiobench_artefacts\Standalone\Audiobench.exe artifacts\Audiobench_Windows_x64_Standalone.exe
cp _build\Audiobench_artefacts\VST3\Audiobench.vst3\Contents\x86_64-windows\Audiobench.dll artifacts\Audiobench_Windows_x64_VST3.dll
