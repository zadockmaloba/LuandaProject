#pragma once

#if !TARGET_OS_OSX
#error "Metal renderer can only be used for OSX targets"
#endif

#import <Metal/Metal.h>

#ifdef __cplusplus
extern "C" {
#endif

void* luanda_renderer_create(id<MTLDevice> device);
void luanda_renderer_render(void* renderer, size_t width, size_t height);
id<MTLTexture> luanda_renderer_get_texture(void* renderer);
void luanda_renderer_destroy(void* renderer);

#ifdef __cplusplus
}
#endif