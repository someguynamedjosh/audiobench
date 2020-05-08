/*
  ==============================================================================

    This file was auto-generated!

    It contains the basic framework code for a JUCE plugin editor.

  ==============================================================================
*/

#include "PluginProcessor.h"
#include "PluginEditor.h"
#include "audiobench.h"

void pushState(void *gp) {
    ((Graphics*) gp)->saveState();
}

void popState(void *gp) {
    ((Graphics*) gp)->restoreState();
}

void applyOffset(void *gp, int x, int y) {
    ((Graphics*) gp)->addTransform(AffineTransform().translated(x, y));
}

void setColor(void *gp, uint8_t r, uint8_t g, uint8_t b) {
    ((Graphics*) gp)->setColour(Colour(r, g, b));
}

void setAlpha_notJuce(void *gp, float alpha) {
    ((Graphics*) gp)->setOpacity(alpha);
}

void clear(void *gp) {
    ((Graphics*) gp)->fillAll();
}

void strokeLine(void *gp, int x1, int y1, int x2, int y2, float weight) {
    ((Graphics*) gp)->drawLine(x1, y1, x2, y2, weight);
}

void fillRect(void *gp, int x, int y, int w, int h) {
    ((Graphics*) gp)->fillRect(x, y, w, h);
}

void fillPie(void *gp, int x, int y, int r, int ir, float sr, float er) {
    Path pie;
    pie.addPieSegment(
        (float) x,
        (float) y,
        (float) r,
        (float) r,
        // JUCE people don't know how math works and made 0 radians up and clockwise positive.
        M_PI_2 - sr,
        M_PI_2 - er,
        ((float) ir) / ((float) r)
    );
    ((Graphics*) gp)->fillPath(pie);
}

void writeLabel(void *gp, int x, int y, int w, char *text) {
    String str = String(text);
    ((Graphics*) gp)->setFont(12.0f);
    ((Graphics*) gp)->drawFittedText(str, x, y, w, 30, Justification::centredTop, 2);
}

//==============================================================================
AudioBenchAudioProcessorEditor::AudioBenchAudioProcessorEditor (AudioBenchAudioProcessor& p)
    : AudioProcessorEditor (&p), processor (p)
{
    ABGraphicsFunctions fns;
    fns.pushState = pushState;
    fns.popState = popState;
    fns.applyOffset = applyOffset;

    fns.setColor = setColor;
    fns.setAlpha = setAlpha_notJuce;
    fns.clear = clear;
    fns.strokeLine = strokeLine;
    fns.fillRect = fillRect;
    fns.fillPie = fillPie;
    fns.writeLabel = writeLabel;
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
