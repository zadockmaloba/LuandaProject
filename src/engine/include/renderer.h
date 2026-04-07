#pragma once

#include <stdint.h>

typedef enum LuandaBackend {
    LUANDA_BACKEND_METAL = 0,
    LUANDA_BACKEND_D3D12 = 1,
    LUANDA_BACKEND_VULKAN = 2,
} LuandaBackend;

typedef struct LuandaTextureHandle {
    LuandaBackend backend;
    void* handle;
} LuandaTextureHandle;

typedef struct LuandaExternalDevice {
    LuandaBackend backend;
    void* device;
} LuandaExternalDevice;

#ifdef __cplusplus
extern "C" {
#endif

void* luanda_renderer_create(int backend, LuandaExternalDevice* external_device);
void  luanda_renderer_draw(void* renderer, uint32_t width, uint32_t height);
int   luanda_renderer_get_texture(void* renderer, LuandaTextureHandle* out_texture);
void  luanda_renderer_destroy(void* renderer);

#ifdef __cplusplus
}
#endif