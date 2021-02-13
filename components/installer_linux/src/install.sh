#!/usr/bin/env sh

echo "Extraction finished, beginning installation..."
echo ""
echo "This installer will copy the following files:"
echo "Audiobench.bin -> /usr/bin/"
echo "Audiobench.vst3 -> /usr/lib/vst3/"
echo "libaudiobench_clib.so -> /usr/lib/"
echo "AudiobenchStandalone.desktop -> /usr/share/applications/"
echo "Julia 1.5 will also be installed, as it is inconsistently packaged across different distros."
echo "If you are asked for your password, it is for permission to copy to these locations."

chmod +x Audiobench.bin
sudo mv ./Audiobench.bin /usr/bin/
chmod +x Audiobench.vst3
sudo mkdir -p /usr/lib/vst3
sudo mv ./Audiobench.vst3 /usr/lib/vst3/
sudo mv ./libaudiobench_clib.so /usr/lib/

sudo rm /usr/lib/libjulia.so*
sudo cp -r ./julia/* /usr/

chmod +x AudiobenchStandalone.desktop
sudo mkdir -p /usr/share/applications/
sudo mv ./AudiobenchStandalone.desktop /usr/share/applications/

echo ""
echo "Installation complete!"
echo "You can now launch the standalone version from your DE's menu or the VST3 version from any compatible DAW."
