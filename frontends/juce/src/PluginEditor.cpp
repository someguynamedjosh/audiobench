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

void applyOffset(void *gp, float x, float y)
{
    ((Graphics *)gp)->addTransform(AffineTransform().translated(x, y));
}

void applyScale(void *gp, float s)
{
    ((Graphics *)gp)->addTransform(AffineTransform().scaled(s));
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

void strokeLine(void *gp, float x1, float y1, float x2, float y2, float weight)
{
    ((Graphics *)gp)->drawLine(x1 - 0.5f, y1 - 0.5f, x2 - 0.5f, y2 - 0.5f, weight);
}

void fillRect(void *gp, float x, float y, float w, float h)
{
    ((Graphics *)gp)->fillRect(x, y, w, h);
}

void fillRoundedRect(void *gp, float x, float y, float w, float h, float cornerSize)
{
    ((Graphics *)gp)->fillRoundedRectangle(x, y, w, h, cornerSize);
}

void fillPie(void *gp, float x, float y, float r, float ir, float sr, float er)
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
    void *gp, float fontSize, float x, float y, float w, float h, uint8_t halign,
    uint8_t valign, int maxLines, char *text)
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

void writeConsoleText(void *gp, float w, float h, char *text)
{
    String str = String(text);

    Font newFont = Font(Font::getDefaultMonospacedFontName(), 14.0, 0);
    ((Graphics *)gp)->setFont(newFont);
    ((Graphics *)gp)->setColour(Colours::white);
    float x = 2, y = 14;
    bool inEscapeCode = false;
    String escapeCode = String("");
    for (auto c : str)
    {
        if (c == '\x1B')
        {
            inEscapeCode = true;
            continue;
        }
        if (inEscapeCode)
        {
            escapeCode.append(String(&c, 1), 1);
            // This is a very hacky implementation of an escape sequence parser.
            if (c >= '\x40' && c <= '\x7E' && c != '[')
            {
                inEscapeCode = false;
                // Change text appearence. We don't bother parsing any others because they aren't
                // useful.
                // Also there's only like 7 colors supported here because it's just writing error
                // messages.
                if (c == 'm')
                {
                    if (escapeCode == "[0m")
                    {
                        ((Graphics *)gp)->setColour(Colours::white);
                    }
                    else if (escapeCode == "[34m")
                    {
                        ((Graphics *)gp)->setColour(Colours::cyan);
                    }
                    else if (escapeCode == "[96m")
                    {
                        ((Graphics *)gp)->setColour(Colours::cyan);
                    }
                    else if (escapeCode == "[31m")
                    {
                        ((Graphics *)gp)->setColour(Colours::darkred);
                    }
                    else if (escapeCode == "[91m")
                    {
                        ((Graphics *)gp)->setColour(Colours::red);
                    }
                    else if (escapeCode == "[33m")
                    {
                        ((Graphics *)gp)->setColour(Colours::gold);
                    }
                    else if (escapeCode == "[93m")
                    {
                        ((Graphics *)gp)->setColour(Colours::yellow);
                    }
                    else
                    {
                        ((Graphics *)gp)->setColour(Colours::magenta);
                    }
                }
                escapeCode.clear();
            }
            continue;
        }
        ((Graphics *)gp)->drawSingleLineText(String(&c, 1), x, y);
        if (c == '\n')
        {
            x = 2;
            y += 14;
        }
        else
        {
            x += 7;
        }
    }
    Font oldFont = Font(Font::getDefaultSansSerifFontName(), 14.0, 0);
    ((Graphics *)gp)->setFont(oldFont);
}

void drawIcon(void *gp, void *iconStore, bool white, int index, float x, float y, float size)
{
    index = index * 2 + (white ? 1 : 0);
    (*(std::vector<std::unique_ptr<Drawable>> *)iconStore)[index]->draw(
        *((Graphics *)gp), 1.0f,
        AffineTransform::scale(size / 24.0f).translated(x, y));
}

void drawDropShadow(void *gp, float x, float y, float w, float h, float radius)
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
AudiobenchAudioProcessorEditor::AudiobenchAudioProcessorEditor(AudiobenchAudioProcessor &p)
    : AudioProcessorEditor(&p), processor(p)
{
    // The Rust library will use these functions to paint to the screen.
    ABGraphicsFunctions fns;
    fns.pushState = pushState;
    fns.popState = popState;
    fns.applyOffset = applyOffset;
    fns.applyScale = applyScale;

    fns.setColor = setColor;
    fns.setAlpha = setAlpha_notJuce;
    fns.clear = clear;
    fns.strokeLine = strokeLine;
    fns.fillRect = fillRect;
    fns.fillRoundedRect = fillRoundedRect;
    fns.fillPie = fillPie;
    fns.writeText = writeText;
    fns.writeConsoleText = writeConsoleText;
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
    addKeyListener(this);
    setWantsKeyboardFocus(true);
    // Our timer method repaints the screen. The number here is then basically the (maximum) FPS
    // that our GUI will run at. Ideally, this should be related to the interval that feedback data
    // is copied from the audio thread to the GUI thread, which can be found in src/engine/base.rs
    startTimerHz(40);
}

AudiobenchAudioProcessorEditor::~AudiobenchAudioProcessorEditor()
{
    ABDestroyUI(processor.ab);
}

//==============================================================================
void AudiobenchAudioProcessorEditor::paint(Graphics &g)
{
    // Rust will pass the pointer to the Graphics object as the first argument to the drawing
    // functions whenever it uses them.
    ABDrawUI(processor.ab, (void *)&g, (void *)&iconStore);
}

void AudiobenchAudioProcessorEditor::mouseDown(const MouseEvent &event)
{
    ABUIMouseDown(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudiobenchAudioProcessorEditor::mouseMove(const MouseEvent &event)
{
    ABUIMouseMove(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudiobenchAudioProcessorEditor::mouseDrag(const MouseEvent &event)
{
    ABUIMouseMove(processor.ab, event.x, event.y, event.mods.isPopupMenu(), event.mods.isShiftDown(), event.mods.isAltDown());
}

void AudiobenchAudioProcessorEditor::mouseUp(const MouseEvent &event)
{
    ABUIMouseUp(processor.ab);
}

bool AudiobenchAudioProcessorEditor::keyPressed(const KeyPress &key, Component *originatingComponent)
{
    ABUIKeyPress(processor.ab, (char)key.getTextCharacter());
    return true;
}

void AudiobenchAudioProcessorEditor::resized()
{
    // This is generally where you'll want to lay out the positions of any
    // subcomponents in your editor..
}
