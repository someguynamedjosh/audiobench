# Audiobench

Audiobench is a free, open-source modular synthesizer. It can be used to create
a variety of sounds by connecting audio processing modules in unlimited ways. It
can be downloaded from [the main website](https://bit.ly/audio_bench), and a
[getting started](https://joshua-maros.github.io/audiobench/book/getting_started.html)
guide is also available.

![Screenshot of a simple patch](docs/book/src/images/default_e.png)

## Building 

First, make sure you have installed all the necessary tools and dependencies:

### Windows Dependencies
- `git`
- `python3`
- `cmake` >= 3.15
- Visual Studio 16 2019 and its build tools
- A Rust toolchain compatible with MSVC (this has been the default since 2017.)
- Run the command
  `git clone "https://gitlab.com/Code_Cube/llvm-win.git" "C:\LLVM` to download
  necessary LLVM tools (the official builds lack some of the tooling necessary
  to compile Audiobench.)
- Set the environment variable `LLVM_SYS_70_PREFIX` to `C:\LLVM`

### MacOS Dependencies
- `git`
- `python3`
- `cmake` >= 3.15
- Xcode toolchain
- Rust toolchain
- `llvm@7` from Homebrew.
- Set the environment variable `LLVM_SYS_70_PREFIX` to
  `/usr/local/Cellar/llvm@7/7.1.0_2`

### Linux Dependencies
- `git`
- `python3`
- `make`
- `gcc` toolchain
- `cmake` >= 3.15
- Rust toolchain
- Required development libraries can be installed on Debian-based systems with
  the command
  `sudo apt -y install llvm-7 libxrandr-dev libxinerama-dev libxcursor-dev libasound-dev extra-cmake-modules libxcb-shape0-dev libxcb-xfixes0-dev`

### Build Process

Before building for the first time, run `git submodule init; git submodule sync`
to pull code for all submodules. The build system is contained in `build.py`.
It can be run by doing `./build.py` or `python3 build.py`, the first form may
not work on Windows. Running it will provide a description of how it can be
used. The most common uses are as follows:
- `./build.py juce_frontend --release` builds a release version of the
  standalone and plugin versions of Audiobench. The results are placed in
  `artifacts/bin/`.
- `./build.py run` builds and runs a debug version.
- `./build.py benchmark --release` runs performance tests and measures how long
  different parts of the code take to run.