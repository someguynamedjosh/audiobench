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

void fillRoundedRect(void *gp, int x, int y, int w, int h, int cornerSize) {
    ((Graphics*) gp)->fillRoundedRectangle(x, y, w, h, cornerSize);
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

void drawIcon(void *gp, void *iconStore, int index, int x, int y, int size) {
    (*(std::vector<std::unique_ptr<Drawable>>*) iconStore)[index]->draw(
        *((Graphics*) gp), 1.0f, 
        AffineTransform::scale(size / 24.0f).translated(x, y)
    );
}

//==============================================================================
AudioBenchAudioProcessorEditor::AudioBenchAudioProcessorEditor (AudioBenchAudioProcessor& p)
    : AudioProcessorEditor (&p), processor (p)
{
    // The Rust library will use these functions to paint to the screen.
    ABGraphicsFunctions fns;
    fns.pushState = pushState;
    fns.popState = popState;
    fns.applyOffset = applyOffset;

    fns.setColor = setColor;
    fns.setAlpha = setAlpha_notJuce;
    fns.clear = clear;
    fns.strokeLine = strokeLine;
    fns.fillRect = fillRect;
    fns.fillRoundedRect = fillRoundedRect;
    fns.fillPie = fillPie;
    fns.writeLabel = writeLabel;
    fns.drawIcon = drawIcon;
    ABSetGraphicsFunctions(p.ab, fns);

    int numIcons = ABGetNumIcons(p.ab);
    for (int index = 0; index < numIcons; index++) {
        void *svgData;
        int dataSize;
        ABGetIconData(p.ab, index, &svgData, &dataSize);
        iconStore.push_back(Drawable::createFromImageData(svgData, dataSize));
    }

    // Make sure that before the constructor has finished, you've set the
    // editor's size to whatever you need it to be.
    setSize (640, 480);
    ABCreateUI(processor.ab);
}

AudioBenchAudioProcessorEditor::~AudioBenchAudioProcessorEditor()
{
    ABDestroyUI(processor.ab);
}

//==============================================================================
void AudioBenchAudioProcessorEditor::paint (Graphics& g)
{
    // Rust will pass the pointer to the Graphics object as the first argument to the drawing 
    // functions whenever it uses them.
    ABDrawUI(processor.ab, (void*) &g, (void*) &iconStore);
}

void AudioBenchAudioProcessorEditor::mouseDown(const MouseEvent &event) {
    ABUIMouseDown(processor.ab, event.x, event.y, event.mods.isPopupMenu());
    repaint();
}

void AudioBenchAudioProcessorEditor::mouseMove(const MouseEvent &event) {
    ABUIMouseMove(processor.ab, event.x, event.y);
    repaint();
}

void AudioBenchAudioProcessorEditor::mouseDrag(const MouseEvent &event) {
    ABUIMouseMove(processor.ab, event.x, event.y);
    repaint();
}

void AudioBenchAudioProcessorEditor::mouseUp(const MouseEvent &event) {
    ABUIMouseUp(processor.ab);
    repaint();
}

void AudioBenchAudioProcessorEditor::resized()
{
    // This is generally where you'll want to lay out the positions of any
    // subcomponents in your editor..
}
