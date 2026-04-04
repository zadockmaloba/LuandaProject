#pragma once
#import <Metal/Metal.h>

extern "C" {
void* luanda_renderer_create(id<MTLDevice> device);
void luanda_renderer_render(void* renderer, size_t width, size_t height);
id<MTLTexture> luanda_renderer_get_texture(void* renderer);
void luanda_renderer_destroy(void* renderer);
}
