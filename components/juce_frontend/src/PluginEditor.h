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
class AudiobenchAudioProcessorEditor  : public AudioProcessorEditor, public Timer, public KeyListener
{
public:
    AudiobenchAudioProcessorEditor (AudiobenchAudioProcessor&);
    ~AudiobenchAudioProcessorEditor();

    //==============================================================================
    void paint (Graphics&) override;
    void resized() override;
    void mouseDown(const MouseEvent &event) override;
    void mouseMove(const MouseEvent &event) override;
    void mouseDrag(const MouseEvent &event) override;
    void mouseUp(const MouseEvent &event) override;
    virtual void mouseWheelMove(const MouseEvent &event, const MouseWheelDetails &wheel) override;
    virtual bool keyPressed(const KeyPress &key, Component *originatingComponent) override;
    void timerCallback() override { 
      repaint(); 
      if (!focusGrabbed && isShowing()) {
        grabKeyboardFocus();
        focusGrabbed = true;
      }
    }

private:
    // This reference is provided as a quick way for your editor to
    // access the processor object that created it.
    AudiobenchAudioProcessor &processor;
    ComponentBoundsConstrainer *constrainer;
    double windowScale = 1.0;
    bool focusGrabbed = false;
    std::vector<std::unique_ptr<Drawable>> iconStore;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR (AudiobenchAudioProcessorEditor)
};
