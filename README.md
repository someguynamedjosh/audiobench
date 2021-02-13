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

### MacOS Dependencies
- `git`
- `python3`
- `cmake` >= 3.15
- Xcode toolchain
- Rust toolchain

### Linux Dependencies
Several tools and libraries are necessary. They can be installed on Debian-based
systems with the following command:
```bash
sudo apt -y install \
  git python3 make gcc cmake \
  libxrandr-dev libxinerama-dev libxcursor-dev libasound-dev libtinfo-dev \
  extra-cmake-modules libxcb-shape0-dev libxcb-xfixes0-dev libclang-dev
```

You must also install the Rust toolchain from [rustup.rs](https://rustup.rs)

Check that your cmake version is at least `3.15` with `cmake --version`.

### Build Process

The build system is contained in `build.py`. It can be run by doing `./build.py`
or `python build.py`, the first form may not work on Windows. Running it will
provide a description of how it can be used. The most common uses are as
follows:
- `./build.py juce_frontend --release` builds a release version of the
  standalone and plugin versions of Audiobench. The results are placed in
  `artifacts/bin/`.
- `./build.py run` builds and runs a debug version.
- `./build.py benchmark --release` runs performance tests and measures how long
  different parts of the code take to run.
The first time you run a build it will take a while to build additional
dependencies that are not reliably available in packaged form.