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
mv macos-installer-builder/macOS-x64/target/pkg/*.pkg ../../artifacts/installer/

rm -rf macos-installer-builder/
