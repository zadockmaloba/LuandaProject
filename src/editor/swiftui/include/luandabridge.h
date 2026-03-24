#pragma once
#import <Metal/Metal.h>

void* luanda_renderer_create(id<MTLDevice> device);
void luanda_renderer_draw(void* renderer, MTLRenderPassDescriptor* descriptor);
void luanda_renderer_destroy(void* renderer);