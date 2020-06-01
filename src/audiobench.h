#pragma once

typedef void *ABInstanceRef;
struct ABGraphicsFunctions {
    void (*pushState)(void*);
    void (*popState)(void*);
    void (*applyOffset)(void*, int, int);
    void (*applyScale)(void*, float);

    void (*setColor)(void*, uint8_t, uint8_t, uint8_t);
    void (*setAlpha)(void*, float);
    void (*clear)(void*);
    void (*strokeLine)(void*, int, int, int, int, float);
    void (*fillRect)(void*, int, int, int, int);
    void (*fillRoundedRect)(void*, int, int, int, int, int);
    void (*fillPie)(void*, int, int, int, int, float, float);
    void (*writeText)(void*, int, int, int, int, int, char, char, int, char*);
    void (*writeConsoleText)(void*, int, int, char*);
    void (*drawIcon)(void*, void*, bool, int, int, int, int);
    void (*drawDropShadow)(void*, int, int, int, int, int);
};

extern "C" {
    ABInstanceRef ABCreateInstance();
    void ABDestroyInstance(ABInstanceRef);
    int ABGetNumIcons(ABInstanceRef);
    void ABGetIconData(ABInstanceRef, int iconIndex, void **dataBufferPtr, int *sizePtr);
    float *ABSetBufferLengthAndSampleRate(ABInstanceRef, int, int);
    void ABNoteOn(ABInstanceRef, int, float);
    void ABNoteOff(ABInstanceRef, int);
    float *ABRenderAudio(ABInstanceRef);
    void ABSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);

    void ABCreateUI(ABInstanceRef);
    void ABDrawUI(ABInstanceRef, void *extraData, void *iconStore);
    void ABDestroyUI(ABInstanceRef);
    void ABUIMouseDown(ABInstanceRef, int, int, bool, bool, bool);
    void ABUIMouseMove(ABInstanceRef, int, int, bool, bool, bool);
    void ABUIMouseUp(ABInstanceRef);
    void ABUIKeyPress(ABInstanceRef, char);
}
