/*
  ==============================================================================

    This file was auto-generated!

    It contains the basic framework code for a JUCE plugin editor.

  ==============================================================================
*/

#include "PluginProcessor.h"
#include "PluginEditor.h"
#include "audiobench.h"

void pushState(void *gp)
{
    ((Graphics *)gp)->saveState();
}

void popState(void *gp)
{
    ((Graphics *)gp)->restoreState();
}

void applyOffset(void *gp, int x, int y)
{
    ((Graphics *)gp)->addTransform(AffineTransform().translated(x, y));
}

void setColor(void *gp, uint8_t r, uint8_t g, uint8_t b)
{
    ((Graphics *)gp)->setColour(Colour(r, g, b));
}

void setAlpha_notJuce(void *gp, float alpha)
{
    ((Graphics *)gp)->setOpacity(alpha);
}

void clear(void *gp)
{
    ((Graphics *)gp)->fillAll();
}

void strokeLine(void *gp, int x1, int y1, int x2, int y2, float weight)
{
    ((Graphics *)gp)->drawLine(x1 - 0.5f, y1 - 0.5f, x2 - 0.5f, y2 - 0.5f, weight);
}

void fillRect(void *gp, int x, int y, int w, int h)
{
    ((Graphics *)gp)->fillRect(x, y, w, h);
}

void fillRoundedRect(void *gp, int x, int y, int w, int h, int cornerSize)
{
    ((Graphics *)gp)->fillRoundedRectangle(x, y, w, h, cornerSize);
}

void fillPie(void *gp, int x, int y, int r, int ir, float sr, float er)
{
    Path pie;
    pie.addPieSegment(
        (float)x,
        (float)y,
        (float)r,
        (float)r,
        // JUCE people don't know how math works and made 0 radians up and clockwise positive.
        M_PI_2 - sr,
        M_PI_2 - er,
        ((float)ir) / ((float)r));
    ((Graphics *)gp)->fillPath(pie);
}

void writeText(
    void *gp, int fontSize, int x, int y, int w, int h, char halign,
    char valign, int maxLines, char *text)
{
    int justification = 0;
    if (halign == 0)
    {
        justification |= Justification::left;
    }
    else if (halign == 1)
    {
        justification |= Justification::horizontallyCentred;
    }
    else if (halign == 2)
    {
        justification |= Justification::right;
    }
    if (valign == 0)
    {
        justification |= Justification::top;
    }
    else if (valign == 1)
    {
        justification |= Justification::verticallyCentred;
    }
    else if (valign == 2)
    {
        justification |= Justification::bottom;
    }
    String str = String(text);
    ((Graphics *)gp)->setFont((float)fontSize);
    ((Graphics *)gp)->drawFittedText(str, x, y, w, h, justification, maxLines);
}

void drawIcon(void *gp, void *iconStore, bool white, int index, int x, int y, int size)
{
    index = index * 2 + (white ? 1 : 0);
    (*(std::vector<std::unique_ptr<Drawable>> *)iconStore)[index]->draw(
        *((Graphics *)gp), 1.0f,
        AffineTransform::scale(size / 24.0f).translated(x, y));
}

void drawDropShadow(void *gp, int x, int y, int w, int h, int radius)
{
    Graphics *g = (Graphics *)gp;
    auto black = Colours::black.withAlpha(0.4f);
    g->setGradientFill(ColourGradient(black, x, y, Colours::transparentBlack, x, y - radius, false));
    g->fillRect(x, y - radius, w, radius);
    g->setGradientFill(ColourGradient(black, x, y, Colours::transparentBlack, x, y - radius, true));
    g->fillRect(x - radius, y - radius, radius, radius);
    g->setGradientFill(ColourGradient(black, x + w, y, Colours::transparentBlack, x + w, y - radius, true));
    g->fillRect(x + w, y - radius, radius, radius);
    g->setGradientFill(ColourGradient(black, x, y + h, Colours::transparentBlack, x, y + h + radius, false));
    g->fillRect(x, y + h, w, radius);
    g->setGradientFill(ColourGradient(black, x, y + h, Colours::transparentBlack, x, y + h + radius, true));
    g->fillRect(x - radius, y + h, radius, radius);
    g->setGradientFill(ColourGradient(black, x + w, y + h, Colours::transparentBlack, x + w, y + h + radius, true));
    g->fillRect(x + w, y + h, radius, radius);
    g->setGradientFill(ColourGradient(black, x, y, Colours::transparentBlack, x - radius, y, false));
    g->fillRect(x - radius, y, radius, h);
    g->setGradientFill(ColourGradient(black, x + w, y, Colours::transparentBlack, x + w + radius, y, false));
    g->fillRect(x + w, y, radius, h);
}

//==============================================================================
AudioBenchAudioProcessorEditor::AudioBenchAudioProcessorEditor(AudioBenchAudioProcessor &p)
    : AudioProcessorEditor(&p), processor(p)
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
    fns.writeText = writeText;
    fns.drawIcon = drawIcon;
    fns.drawDropShadow = drawDropShadow;
    ABSetGraphicsFunctions(p.ab, fns);

    int numIcons = ABGetNumIcons(p.ab);
    for (int index = 0; index < numIcons; index++)
    {
        void *svgData;
        int dataSize;
        ABGetIconData(p.ab, index, &svgData, &dataSize);
        {
            iconStore.push_back(Drawable::createFromImageData(svgData, dataSize));
        }
        {
            iconStore.push_back(Drawable::createFromImageData(svgData, dataSize));
            iconStore.back()->replaceColour(Colours::black, Colours::white);
        }
    }

    // Make sure that before the constructor has finished, you've set the
    // editor's size to whatever you need it to be.
    setSize(640, 480);
    ABCreateUI(processor.ab);
    // Our timer method repaints the screen. The number here is then basically the (maximum) FPS
    // that our GUI will run at. Ideally, this should be related to the interval that feedback data
    // is copied from the audio thread to the GUI thread, which can be found in src/engine/base.rs
    startTimerHz(40);
}

AudioBenchAudioProcessorEditor::~AudioBenchAudioProcessorEditor()
{
    ABDestroyUI(processor.ab);
}

//==============================================================================
void AudioBenchAudioProcessorEditor::paint(Graphics &g)
{
    // Rust will pass the pointer to the Graphics object as the first argument to the drawing
    // functions whenever it uses them.
    ABDrawUI(processor.ab, (void *)&g, (void *)&iconStore);
}

void AudioBenchAudioProcessorEditor::mouseDown(const MouseEvent &event)
{
    ABUIMouseDown(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudioBenchAudioProcessorEditor::mouseMove(const MouseEvent &event)
{
    ABUIMouseMove(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudioBenchAudioProcessorEditor::mouseDrag(const MouseEvent &event)
{
    ABUIMouseMove(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudioBenchAudioProcessorEditor::mouseUp(const MouseEvent &event)
{
    ABUIMouseUp(processor.ab);
}

void AudioBenchAudioProcessorEditor::resized()
{
    // This is generally where you'll want to lay out the positions of any
    // subcomponents in your editor..
}
