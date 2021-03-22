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
    int ABUiGetNumIcons(ABInstanceRef);
    void ABUiGetIconData(ABInstanceRef, int iconIndex, void **dataBufferPtr, int *sizePtr);
    float *ABAudioSetGlobalParameters(ABInstanceRef, int, int);

    void ABAudioSerializePatch(ABInstanceRef, char**, uint32_t*);
    void ABUiSerializePatch(ABInstanceRef, char**, uint32_t*);
    void ABCleanupSerializedData(char*, uint32_t);
    void ABAudioDeserializePatch(ABInstanceRef, char*, uint32_t);
    void ABUiDeserializePatch(ABInstanceRef, char*, uint32_t);
    void ABUiHandleCrossThreadHelp(ABInstanceRef);

    void ABAudioStartNote(ABInstanceRef, int, float);
    void ABAudioReleaseNote(ABInstanceRef, int);
    void ABAudioPitchWheel(ABInstanceRef, float);
    void ABAudioBpm(ABInstanceRef, float);
    void ABAudioSongTime(ABInstanceRef, float);
    void ABAudioSongBeats(ABInstanceRef, float);
    void ABAudioControl(ABInstanceRef, int, float);
    float *ABAudioRenderAudio(ABInstanceRef);

    void ABUiSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);
    void ABUiCreateUI(ABInstanceRef);
    void ABUiDrawUI(ABInstanceRef, void *extraData, void *iconStore);
    void ABUiDestroyUI(ABInstanceRef);
    void ABUiMouseDown(ABInstanceRef, float, float, bool, bool, bool);
    void ABUiMouseMove(ABInstanceRef, float, float, bool, bool, bool);
    void ABUiMouseUp(ABInstanceRef);
    void ABUiScroll(ABInstanceRef, float);
    void ABUiKeyPress(ABInstanceRef, char);
    void ABUiKeyDown(ABInstanceRef, char);
    void ABUiKeyUp(ABInstanceRef, char);
}
