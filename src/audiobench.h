#pragma once

typedef void *ABInstanceRef;
struct ABGraphicsFunctions {
    void (*pushState)(void*);
    void (*popState)(void*);
    void (*applyOffset)(void*, int, int);

    void (*setColor)(void*, uint8_t, uint8_t, uint8_t);
    void (*setAlpha)(void*, float);
    void (*clear)(void*);
    void (*strokeLine)(void*, int, int, int, int, float);
    void (*fillRect)(void*, int, int, int, int);
    void (*fillRoundedRect)(void*, int, int, int, int, int);
    void (*fillPie)(void*, int, int, int, int, float, float);
    void (*writeLabel)(void*, int, int, int, char*);
};

extern "C" {
    ABInstanceRef ABCreateInstance();
    void ABDestroyInstance(ABInstanceRef);
    float *ABRenderAudio(ABInstanceRef);
    void ABSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);

    void ABCreateUI(ABInstanceRef);
    void ABDrawUI(ABInstanceRef, void *extraData);
    void ABDestroyUI(ABInstanceRef);
    void ABUIMouseDown(ABInstanceRef, int, int, bool);
    void ABUIMouseMove(ABInstanceRef, int, int);
    void ABUIMouseUp(ABInstanceRef);

    void ABGetSvgData(void **dataBufferPtr, int *sizePtr);
}
