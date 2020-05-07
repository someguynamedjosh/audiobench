#pragma once

typedef void *ABTestStructRef;

extern "C" {
    ABTestStructRef ABCreateTestStruct();
    void ABDestroyTestStruct(ABTestStructRef);
    float ABAttenuate(ABTestStructRef, float);
}
