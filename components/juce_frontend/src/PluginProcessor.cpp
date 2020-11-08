/*
  ==============================================================================

    This file was auto-generated!

    It contains the basic framework code for a JUCE plugin processor.

  ==============================================================================
*/

#include "PluginProcessor.h"

#include <JuceHeader.h>

#include "PluginEditor.h"
#include "audiobench.h"

//==============================================================================
AudiobenchAudioProcessor::AudiobenchAudioProcessor()
#ifndef JucePlugin_PreferredChannelConfigurations
    : AudioProcessor(BusesProperties()
#if !JucePlugin_IsMidiEffect
#if !JucePlugin_IsSynth
                         .withInput("Input", AudioChannelSet::stereo(), true)
#endif
                         .withOutput("Output", AudioChannelSet::stereo(), true)
#endif
      )
#endif
{
    ab = ABCreateInstance();
}

AudiobenchAudioProcessor::~AudiobenchAudioProcessor() { ABDestroyInstance(ab); }

//==============================================================================
const String AudiobenchAudioProcessor::getName() const {
    return JucePlugin_Name;
}

bool AudiobenchAudioProcessor::acceptsMidi() const {
#if JucePlugin_WantsMidiInput
    return true;
#else
    return false;
#endif
}

bool AudiobenchAudioProcessor::producesMidi() const {
#if JucePlugin_ProducesMidiOutput
    return true;
#else
    return false;
#endif
}

bool AudiobenchAudioProcessor::isMidiEffect() const {
#if JucePlugin_IsMidiEffect
    return true;
#else
    return false;
#endif
}

double AudiobenchAudioProcessor::getTailLengthSeconds() const { return 0.0; }

int AudiobenchAudioProcessor::getNumPrograms() {
    return 1;  // NB: some hosts don't cope very well if you tell them there are
               // 0 programs, so this should be at least 1, even if you're not
               // really implementing programs.
}

int AudiobenchAudioProcessor::getCurrentProgram() { return 0; }

void AudiobenchAudioProcessor::setCurrentProgram(int index) {}

const String AudiobenchAudioProcessor::getProgramName(int index) { return {}; }

void AudiobenchAudioProcessor::changeProgramName(int index,
                                                 const String& newName) {}

//==============================================================================
void AudiobenchAudioProcessor::prepareToPlay(double sampleRate,
                                             int samplesPerBlock) {
    // Use this method as the place to do any pre-playback
    // initialisation that you need..
    ABSetHostFormat(ab, samplesPerBlock, (int)sampleRate);
}

void AudiobenchAudioProcessor::releaseResources() {
    // When playback stops, you can use this as an opportunity to free up any
    // spare memory, etc.
}

#ifndef JucePlugin_PreferredChannelConfigurations
bool AudiobenchAudioProcessor::isBusesLayoutSupported(
    const BusesLayout& layouts) const {
#if JucePlugin_IsMidiEffect
    ignoreUnused(layouts);
    return true;
#else
    // This is the place where you check if the layout is supported.
    // In this template code we only support mono or stereo.
    if (layouts.getMainOutputChannelSet() != AudioChannelSet::mono() &&
        layouts.getMainOutputChannelSet() != AudioChannelSet::stereo())
        return false;

        // This checks if the input layout matches the output layout
#if !JucePlugin_IsSynth
    if (layouts.getMainOutputChannelSet() != layouts.getMainInputChannelSet())
        return false;
#endif

    return true;
#endif
}
#endif

void AudiobenchAudioProcessor::processBlock(AudioBuffer<float>& buffer,
                                            MidiBuffer& midiMessages) {
    ScopedNoDenormals noDenormals;
    auto totalNumInputChannels = getTotalNumInputChannels();
    auto totalNumOutputChannels = getTotalNumOutputChannels();

    // Doing two seperate loops prevents the problem where a note is turned on
    // and off in the same buffer, but the on is processed after the off so the
    // note just stays on forever.
    for (auto meta : midiMessages) {
        auto message = meta.getMessage();
        if (message.isNoteOn()) {
            ABStartNote(ab, message.getNoteNumber(),
                        message.getFloatVelocity());
        } else if (message.isPitchWheel()) {
            float value = (message.getPitchWheelValue() - 0x2000 + 0.5f) /
                          (0x2000 - 0.5f);
            ABPitchWheel(ab, value);
        } else if (message.isController()) {
            float value =
                (message.getControllerValue() - 0x40 + 0.5f) / (0x40 - 0.5f);
            ABControl(ab, message.getControllerNumber(), value);
        }
    }
    for (auto meta : midiMessages) {
        auto message = meta.getMessage();
        if (message.isNoteOff()) {
            ABReleaseNote(ab, message.getNoteNumber());
        }
    }
    // MIDI seems to do weird things, this may be helpful in the future.
    // if (midiMessages.getNumEvents() > 0) {
    //     std::cout << "=========" << std::endl;
    // }
    // for (auto meta : midiMessages) {
    //     auto message = meta.getMessage();
    //     if (message.isNoteOn()) {
    //         std::cout << "On " << message.getNoteNumber() << std::endl;
    //     } else if (message.isNoteOff()) {
    //         std::cout << "Off " << message.getNoteNumber() << std::endl;
    //     } else if (message.isPitchWheel()) {
    //         std::cout << "Pitch " << message.getPitchWheelValue() <<
    //         std::endl;
    //     } else if (message.isChannelPressure()) {
    //         std::cout << "Pressure "
    //             << message.getChannel() << " " <<
    //             message.getChannelPressureValue() << std::endl;
    //     } else if (message.isController()) {
    //         std::cout << "Controller "
    //             << message.getControllerNumber() << " " <<
    //             message.getControllerValue() << std::endl;
    //     } else {
    //         std::cout << "Weird message" << std::endl;
    //     }
    // }

    // In case we have more outputs than inputs, this code clears any output
    // channels that didn't contain input data, (because these aren't
    // guaranteed to be empty - they may contain garbage).
    // This is here to avoid people getting screaming feedback
    // when they first compile a plugin, but obviously you don't need to keep
    // this code if your algorithm always overwrites all the output channels.
    for (auto i = totalNumInputChannels; i < totalNumOutputChannels; ++i)
        buffer.clear(i, 0, buffer.getNumSamples());

    float* audioBuffer = ABRenderAudio(ab);

    // This is the place where you'd normally do the guts of your plugin's
    // audio processing...
    // Make sure to reset the state if your inner loop is processing
    // the samples and the outer loop is handling the channels.
    // Alternatively, you can process the samples with the channels
    // interleaved by keeping the same state.
    if (audioBuffer != nullptr) {
        for (int channel = 0; channel < totalNumOutputChannels; ++channel) {
            auto* channelData = buffer.getWritePointer(channel);
            for (int sample = 0; sample < buffer.getNumSamples(); sample++) {
                channelData[sample] = audioBuffer[sample * 2 + channel];
            }
        }
    }
}

//==============================================================================
bool AudiobenchAudioProcessor::hasEditor() const {
    return true;  // (change this to false if you choose to not supply an
                  // editor)
}

AudioProcessorEditor* AudiobenchAudioProcessor::createEditor() {
    return new AudiobenchAudioProcessorEditor(*this);
}

//==============================================================================
void AudiobenchAudioProcessor::getStateInformation(MemoryBlock& destData) {
    char* dataPtr;
    uint32_t dataLen;
    ABSerializePatch(ab, &dataPtr, &dataLen);
    destData.append((void*)dataPtr, dataLen);
    ABCleanupSerializedData(dataPtr, dataLen);
}

void AudiobenchAudioProcessor::setStateInformation(const void* data,
                                                   int sizeInBytes) {
    ABDeserializePatch(ab, (char*)data, sizeInBytes);
}

//==============================================================================
// This creates new instances of the plugin..
AudioProcessor* JUCE_CALLTYPE createPluginFilter() {
    return new AudiobenchAudioProcessor();
}
