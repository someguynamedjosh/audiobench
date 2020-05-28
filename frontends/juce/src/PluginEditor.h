/*
  ==============================================================================

    This file was auto-generated!

    It contains the basic framework code for a JUCE plugin editor.

  ==============================================================================
*/

#pragma once

#include <JuceHeader.h>
#include "PluginProcessor.h"

//==============================================================================
/**
*/
class AudioBenchAudioProcessorEditor  : public AudioProcessorEditor, public Timer, public KeyListener
{
public:
    AudioBenchAudioProcessorEditor (AudioBenchAudioProcessor&);
    ~AudioBenchAudioProcessorEditor();

    //==============================================================================
    void paint (Graphics&) override;
    void resized() override;
    void mouseDown(const MouseEvent &event) override;
    void mouseMove(const MouseEvent &event) override;
    void mouseDrag(const MouseEvent &event) override;
    void mouseUp(const MouseEvent &event) override;
    virtual bool keyPressed(const KeyPress &key, Component *originatingComponent) override;
    void timerCallback() override { 
      repaint(); 
      if (!focusGrabbed && isShowing()) {
        std::cout << "Grabbed!" << std::endl;
        grabKeyboardFocus();
        focusGrabbed = true;
      }
    }

private:
    // This reference is provided as a quick way for your editor to
    // access the processor object that created it.
    AudioBenchAudioProcessor& processor;
    bool focusGrabbed = false;
    std::vector<std::unique_ptr<Drawable>> iconStore;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR (AudioBenchAudioProcessorEditor)
};
