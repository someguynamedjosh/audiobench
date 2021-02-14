#!/usr/bin/env sh

rm -rf macos-installer-builder/ || true

git clone https://github.com/KosalaHerath/macos-installer-builder.git
ARTIFACTS="../../artifacts/bin"
TARGET="macos-installer-builder/macOS-x64/application"

cp -r src/* macos-installer-builder/macOS-x64/darwin/
cp -r $ARTIFACTS/Audiobench_MacOS_x64_Standalone.app $TARGET/Audiobench.app
cp -r $ARTIFACTS/Audiobench_MacOS_x64_VST3.vst3 $TARGET/Audiobench.vst3
cp -r $ARTIFACTS/Audiobench_MacOS_x64_AU.component $TARGET/Audiobench.component
cp -r ../../dependencies/julia $TARGET/Julia.app
cp $ARTIFACTS/libaudiobench_clib.dylib $TARGET/

cd macos-installer-builder/macOS-x64/
yes n | ./build-macos-x64.sh Audiobench $CRATE_VERSION
mkdir -p ../../artifacts/installer/
mv target/pkg/*.pkg ../../artifacts/installer/Audiobench.pkg
echo "1"
ls ../../artifacts/installer/
echo "2"
ls target/pkg/
cp target/pkg/*.pkg ../../artifacts/installer/Audiobench.pkg
echo "3"
ls ../../artifacts/installer/
echo "4"
ls target/pkg/

rm -rf macos-installer-builder/
