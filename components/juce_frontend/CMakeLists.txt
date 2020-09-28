# https://github.com/juce-framework/JUCE/blob/juce6/examples/CMake/AudioPlugin/CMakeLists.txt
# Example Audio Plugin CMakeLists.txt

cmake_minimum_required(VERSION 3.15)

if (${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
    set(MACOS TRUE)
endif()

# Statically link c++ runtime because installing it is a pain
if (WIN32)
    cmake_policy(SET CMP0091 NEW)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
endif()

# The top-level CMakeLists.txt file for a project must contain a literal, direct call to the
# `project()` command. `project()` sets up some helpful variables that describe source/binary
# directories, and the current project version. This is a standard CMake command.

project(AUDIO_PLUGIN_EXAMPLE VERSION 0.0.1)

# If you've installed JUCE somehow (via a package manager, or directly using the CMake install
# target), you'll need to tell this project that it depends on the installed copy of JUCE. If you've
# included JUCE directly in your source tree (perhaps as a submodule), you'll need to tell CMake to
# include that subdirectory as part of the build.

add_subdirectory(juce_git)

# If you are building a VST2 or AAX plugin, CMake needs to be told where to find these SDKs on your
# system. This setup should be done before calling `juce_add_plugin`.

# juce_set_vst2_sdk_path(...)
# juce_set_aax_sdk_path(...)

# `juce_add_plugin` adds a static library target with the name passed as the first argument
# (AudioPluginExample here). This target is a normal CMake target, but has a lot of extra properties set
# up by default. As well as this shared code static library, this function adds targets for each of
# the formats specified by the FORMATS arguments. This function accepts many optional arguments.
# Check the readme at `examples/CMake` in the JUCE repo for the full list.

juce_add_plugin(
    Audiobench 
    # VERSION ...                               # Set this if the plugin version is different to the project version
    # ICON_BIG ...                              # ICON_* arguments specify a path to an image file to use as an icon for the Standalone
    # ICON_SMALL ...
    # COMPANY_NAME ...                          # Specify the name of the plugin's author
    # IS_SYNTH TRUE/FALSE                       # Is this a synth or an effect?
    # NEEDS_MIDI_INPUT TRUE/FALSE               # Does the plugin need midi input?
    # NEEDS_MIDI_OUTPUT TRUE/FALSE              # Does the plugin need midi output?
    # IS_MIDI_EFFECT TRUE/FALSE                 # Is this plugin a MIDI effect?
    # EDITOR_WANTS_KEYBOARD_FOCUS TRUE/FALSE    # Does the editor need keyboard focus?
    # COPY_PLUGIN_AFTER_BUILD TRUE/FALSE        # Should the plugin be installed to a default location after building?
    PLUGIN_MANUFACTURER_CODE Juce               # A four-character manufacturer id with at least one upper-case character
    PLUGIN_CODE Dem0                            # A unique four-character plugin id with at least one upper-case character
    FORMATS AU VST3 Standalone                  # The formats to build. Other valid formats are: AAX Unity VST AU AUv3
    PRODUCT_NAME "Audiobench"                   # The name of the final executable, which can differ from the target name
)

target_include_directories(
    Audiobench PRIVATE
    "$ENV{PROJECT_ROOT}/components/audiobench_clib/"
)

# `juce_generate_juce_header` will create a JuceHeader.h for a given target, which will be generated
# into your build tree. This should be included with `#include <JuceHeader.h>`. The include path for
# this header will be automatically added to the target. The main function of the JuceHeader is to
# include all your JUCE module headers; if you're happy to include module headers directly, you
# probably don't need to call this.

juce_generate_juce_header(Audiobench)

# `target_sources` adds source files to a target. We pass the target that needs the sources as the
# first argument, then a visibility parameter for the sources (PRIVATE is normally best practice,
# although it doesn't really affect executable targets). Finally, we supply a list of source files
# that will be built into the target. This is a standard CMake command.

target_sources(
    Audiobench PRIVATE
    src/PluginEditor.cpp
    src/PluginProcessor.cpp
)

# `target_compile_definitions` adds some preprocessor definitions to our target. In a Projucer
# project, these might be passed in the 'Preprocessor Definitions' field. JUCE modules also make use
# of compile definitions to switch certain features on/off, so if there's a particular feature you
# need that's not on by default, check the module header for the correct flag to set here. These
# definitions will be visible both to your code, and also the JUCE module code, so for new
# definitions, pick unique names that are unlikely to collide! This is a standard CMake command.

target_compile_definitions(
    Audiobench 
    PUBLIC
    JUCE_WEB_BROWSER=0  
    JUCE_USE_CURL=0     
    JUCE_VST3_CAN_REPLACE_VST2=0
)

# If your target needs extra binary assets, you can add them here. The first argument is the name of
# a new static library target that will include all the binary resources. There is an optional
# `NAMESPACE` argument that can specify the namespace of the generated binary data class. Finally,
# the SOURCES argument should be followed by a list of source files that should be built into the
# static library. These source files can be of any kind (wav data, images, fonts, icons etc.).
# Conversion to binary-data will happen when your target is built.

# juce_add_binary_data(Audiobench SOURCES ...)

# `target_link_libraries` links libraries and JUCE modules to other libraries or executables. Here,
# we're linking our executable target to the `juce::juce_audio_utils` module. Inter-module
# dependencies are resolved automatically, so `juce_core`, `juce_events` and so on will also be
# linked automatically. If we'd generated a binary data target above, we would need to link to it
# here too. This is a standard CMake command.

if(WIN32)
    # Statically link visual c++ runtime so it doesn't inexplicably crash if the user
    # hasn't installed msvcrt on their system.
    set_property(TARGET Audiobench PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
    # And of course those changes don't propogate to the other targets because that would be too 
    # easy. Don't worry, I've already made the necessary blood sacrifices to the daemons of 
    # hard-coding and code duplication.
    set_property(TARGET Audiobench_Standalone PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
    set_property(TARGET Audiobench_VST3 PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
    # And yet something still tries to drag in the dynamically-linked library.
    set_property(TARGET Audiobench PROPERTY LINK_FLAGS "/NODEFAULTLIB:MSVCRT")
    set_property(TARGET Audiobench_Standalone PROPERTY LINK_FLAGS "/NODEFAULTLIB:MSVCRT")
    set_property(TARGET Audiobench_VST3 PROPERTY LINK_FLAGS "/NODEFAULTLIB:MSVCRT")

    target_link_libraries(
        Audiobench PRIVATE
        # AudioPluginData           # If we'd created a binary data target, we'd link to it here
        juce::juce_audio_utils
        "$ENV{RUST_OUTPUT_DIR}/audiobench_clib.lib"
        Crypt32
        Ncrypt
        Secur32
        ws2_32
        userenv
    )
elseif(APPLE)
    find_package(CURL REQUIRED)
    find_library(SECURITY_LIBRARY Security)
    target_link_libraries(
        Audiobench PRIVATE
        # AudioPluginData           # If we'd created a binary data target, we'd link to it here
        juce::juce_audio_utils
        "$ENV{RUST_OUTPUT_DIR}/libaudiobench_clib.a"
        ${CURL_LIBRARIES}
        ${SECURITY_LIBRARY}
        ffi
        tinfo
        z
    )
else()
    # The clipboard library I'm using uses a weird X11 wrapper to do its dirty work making this 
    # whole blob necessary.
    find_package(ECM REQUIRED NO_MODULE)
    set(CMAKE_MODULE_PATH ${ECM_FIND_MODULE_DIR})
    find_package(XCB REQUIRED COMPONENTS XCB RENDER SHAPE XFIXES)

    set(THREADS_PREFER_PTHREAD_FLAG ON)
    find_package(Threads REQUIRED)

    target_link_libraries(
        Audiobench PRIVATE
        # AudioPluginData           # If we'd created a binary data target, we'd link to it here
        juce::juce_audio_utils
        "$ENV{RUST_OUTPUT_DIR}/libaudiobench_clib.a"
        z
        ssl
        crypto
        ffi
        ${XCB_LIBRARIES}
        tinfo
        Threads::Threads
    )
endif()
