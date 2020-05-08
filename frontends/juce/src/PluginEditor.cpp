/*
  ==============================================================================

    This file was auto-generated!

    It contains the basic framework code for a JUCE plugin editor.

  ==============================================================================
*/

#include "PluginProcessor.h"
#include "PluginEditor.h"
#include "audiobench.h"

void setColor(void *gp, uint8_t r, uint8_t g, uint8_t b) {
    ((Graphics*) gp)->setColour(Colour(r, g, b));
}

void clear(void *gp) {
    ((Graphics*) gp)->fillAll();
}

void fillRect(void *gp, int x, int y, int w, int h) {
    ((Graphics*) gp)->fillRect(x, y, w, h);
}

void fillPie(void *gp, int x, int y, int r, int ir, float sr, float er) {
    Path pie;
    pie.addPieSegment(
        (float) (x - r),
        (float) (y - r),
        (float) (r * 2),
        (float) (r * 2),
        // JUCE people don't know how math works and made 0 radians up and clockwise positive.
        M_PI_2 - sr,
        M_PI_2 - er,
        ((float) ir) / ((float) r)
    );
    ((Graphics*) gp)->fillPath(pie);
}

//==============================================================================
AudioBenchAudioProcessorEditor::AudioBenchAudioProcessorEditor (AudioBenchAudioProcessor& p)
    : AudioProcessorEditor (&p), processor (p)
{
    ABGraphicsFunctions fns;
    fns.setColor = setColor;
    fns.clear = clear;
    fns.fillRect = fillRect;
    fns.fillPie = fillPie;
    ABSetGraphicsFunctions(p.ab, fns);
    // Make sure that before the constructor has finished, you've set the
    // editor's size to whatever you need it to be.
    setSize (400, 300);
}

AudioBenchAudioProcessorEditor::~AudioBenchAudioProcessorEditor()
{
}

//==============================================================================
void AudioBenchAudioProcessorEditor::paint (Graphics& g)
{
    // (Our component is opaque, so we must completely fill the background with a solid colour)
    g.fillAll (getLookAndFeel().findColour (ResizableWindow::backgroundColourId));

    ABDrawUI(processor.ab, (void*) &g);

    g.setColour (Colours::white);
    g.setFont (15.0f);
    g.drawFittedText ("Hello World!", getLocalBounds(), Justification::centred, 1);
}

void AudioBenchAudioProcessorEditor::resized()
{
    // This is generally where you'll want to lay out the positions of any
    // subcomponents in your editor..
}
