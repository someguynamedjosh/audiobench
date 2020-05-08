#pragma once

typedef void *ABInstanceRef;
struct ABGraphicsFunctions {
    void (*setColor)(void*, uint8_t, uint8_t, uint8_t);
    void (*clear)(void*);
    void (*fillRect)(void*, int, int, int, int);
};

extern "C" {
    ABInstanceRef ABCreateInstance();
    void ABDestroyInstance(ABInstanceRef);
    void ABSetGraphicsFunctions(ABInstanceRef, ABGraphicsFunctions);
    void ABDrawUI(ABInstanceRef, void *extraData);
}
