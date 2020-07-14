#pragma once

typedef void *ABInstanceRef;
struct ABGraphicsFunctions {
    void (*pushState)(void*);
    void (*popState)(void*);
    void (*applyOffset)(void*, float, float);
    void (*applyScale)(void*, float);

    void (*setColor)(void*, uint8_t, uint8_t, uint8_t);
    void (*setAlpha)(void*, float);
    void (*clear)(void*);
    void (*strokeLine)(void*, float, float, float, float, float);
    void (*fillRect)(void*, float, float, float, float);
    void (*fillRoundedRect)(void*, float, float, float, float, float);
    void (*fillPie)(void*, float, float, float, float, float, float);
    void (*writeText)(void*, float, float, float, float, float, uint8_t, uint8_t, int, char*);
    void (*writeConsoleText)(void*, float, float, char*);
    void (*drawIcon)(void*, void*, bool, int, float, float, float);
    void (*drawDropShadow)(void*, float, float, float, float, float);
};

extern "C" {
    ABInstanceRef ABCreateInstance();
    void ABDestroyInstance(ABInstanceRef);
    int ABGetNumIcons(ABInstanceRef);
    void ABGetIconData(ABInstanceRef, int iconIndex, void **dataBufferPtr, int *sizePtr);
    float *ABSetHostFormat(ABInstanceRef, int, int);
    void ABSerializePatch(ABInstanceRef, char**, uint*);
    void ABCleanupSerializedData(char*, uint);
    void ABDeserializePatch(ABInstanceRef, char*, uint);
    void ABStartNote(ABInstanceRef, int, float);
    void ABReleaseNote(ABInstanceRef, int);
    void ABPitchWheel(ABInstanceRef, float);
    void ABBpm(ABInstanceRef, float);
    void ABSongTime(ABInstanceRef, float);
    void ABSongBeats(ABInstanceRef, float);
    void ABControl(ABInstanceRef, int, float);
    float *ABRenderAudio(ABInstanceRef);
    void ABSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);

    void ABCreateUI(ABInstanceRef);
    void ABDrawUI(ABInstanceRef, void *extraData, void *iconStore);
    void ABDestroyUI(ABInstanceRef);
    void ABUIMouseDown(ABInstanceRef, float, float, bool, bool, bool);
    void ABUIMouseMove(ABInstanceRef, float, float, bool, bool, bool);
    void ABUIMouseUp(ABInstanceRef);
    void ABUIScroll(ABInstanceRef, float);
    void ABUIKeyPress(ABInstanceRef, char);
}
