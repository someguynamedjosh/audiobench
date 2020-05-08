#pragma once

typedef void *ABInstanceRef;
struct ABGraphicsFunctions {
    void (*setColor)(void*, uint8_t, uint8_t, uint8_t);
    void (*setAlpha)(void*, float);
    void (*clear)(void*);
    void (*strokeLine)(void*, int, int, int, int, float);
    void (*fillRect)(void*, int, int, int, int);
    void (*fillPie)(void*, int, int, int, int, float, float);
    void (*writeLabel)(void*, int, int, int, char*);
};

extern "C" {
    ABInstanceRef ABCreateInstance();
    void ABDestroyInstance(ABInstanceRef);
    void ABSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);
    void ABDrawUI(ABInstanceRef, void *extraData);
}
