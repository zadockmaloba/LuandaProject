# ImGui Metal Renderer Integration Guide

## Overview

This document explains how to integrate the Rust Metal renderer into the ImGui-based editor. **Note**: The SwiftUI approach cannot be directly copied because SwiftUI and ImGui have fundamentally different rendering architectures.

### Why the Direct Approach Fails

The SwiftUI `MetalView` owns the entire rendering process - it creates the command buffer, presents the drawable, and commits. ImGui also expects to own the rendering process. When both try to present the same drawable, you get the error:

```
Each CAMetalLayerDrawable can only be presented once!
```

### Solution: Three Approaches

1. **Option A**: Render to texture - Game renders to an offscreen texture, ImGui displays it
2. **Option B**: Multi-pass rendering - Separate render passes (complex, not recommended)
3. **Option C**: Shared encoder - Rust renderer uses ImGui's command buffer (recommended)

## Current Architecture

### SwiftUI Implementation
- Uses `MTKView` wrapped in a SwiftUI `NSViewRepresentable`
- `MetalViewCoordinator` implements `MTKViewDelegate`
- Coordinator owns the Rust renderer (`UnsafeMutableRawPointer`)
- Renderer lifecycle: create in `init()`, destroy in `deinit()`
- Drawing: `draw(in:)` calls `luanda_renderer_draw()`

### ImGui Implementation (Current)
- Already has MTKView infrastructure in `osx_main.mm`
- `AppViewController` implements `MTKViewDelegate`
- Has Metal device and command queue
- Currently only renders ImGui UI

## Integration Steps

### 1. Import the Bridge Header

Add to the top of `src/editor/imgui/osx_main.mm`:

```objc
#import "luandabridge.h"
```

### 2. Add Renderer Property

In the `@interface AppViewController` declaration, add:

```objc
@property (nonatomic) void* rustRenderer;
```

### 3. Initialize the Renderer

In `initWithNibName:bundle:`, after creating the Metal device and command queue:

```objc
_device = MTLCreateSystemDefaultDevice();
_commandQueue = [_device newCommandQueue];

// Create the Rust renderer
_rustRenderer = luanda_renderer_create(_device);

if (!self.device)
{
    NSLog(@"Metal is not supported");
    abort();
}
```

### 4. Clean Up the Renderer

In the `dealloc` method, add cleanup at the beginning:

```objc
-(void)dealloc
{
    // Destroy Rust renderer first
    luanda_renderer_destroy(self.rustRenderer);
    
    [super dealloc];
    ImGui_ImplMetal_Shutdown();
#if TARGET_OS_OSX
    ImGui_ImplOSX_Shutdown();
#endif
    if (self.context)
    {
        NSLog(@"Cleaning up Dear ImGui context.");
        ImGui::DestroyContext(self.context);
        self.context = nullptr;
    }
}
```

### 5. Modify Rust Renderer to Not Create Command Buffer

**IMPORTANT**: The current `luanda_renderer_draw()` creates its own command buffer internally, which conflicts with ImGui's rendering. You need to modify the Rust renderer to accept an encoder instead.

#### Option A: Render to Texture (Recommended for ImGui)

Create a new bridge function that renders to a texture:

```c
// In luandabridge.h
id<MTLTexture> luanda_renderer_render_to_texture(void* renderer, id<MTLDevice> device, int width, int height);
```

Then display this texture in an ImGui window:

```objc
-(void)drawInMTKView:(MTKView*)view
{
    // ... setup code ...
    
    ImGui::NewFrame();
    
    // Create viewport window
    ImGui::Begin("Game Viewport");
    
    // Render game to texture
    id<MTLTexture> gameTexture = luanda_renderer_render_to_texture(
        self.rustRenderer, 
        self.device, 
        view.bounds.size.width,
        view.bounds.size.height
    );
    
    // Display texture in ImGui
    ImGui::Image((void*)gameTexture, ImVec2(view.bounds.size.width, view.bounds.size.height));
    ImGui::End();
    
    // ... rest of ImGui rendering ...
}
```

#### Option B: Multi-Pass Rendering (Current Architecture)

Render the game in a separate pass before ImGui:

```objc
-(void)drawInMTKView:(MTKView*)view
{
    ImGuiIO& io = ImGui::GetIO();
    io.DisplaySize.x = view.bounds.size.width;
    io.DisplaySize.y = view.bounds.size.height;

    CGFloat framebufferScale = view.window.screen.backingScaleFactor ?: NSScreen.mainScreen.backingScaleFactor;
    io.DisplayFramebufferScale = ImVec2(framebufferScale, framebufferScale);

    // *** PASS 1: Render game viewport to drawable ***
    luanda_renderer_draw(self.rustRenderer, view.currentRenderPassDescriptor, view.currentDrawable);
    
    // Wait for next drawable for ImGui pass
    id<MTLCommandBuffer> commandBuffer = [self.commandQueue commandBuffer];
    MTLRenderPassDescriptor* renderPassDescriptor = view.currentRenderPassDescriptor;
    
    if (renderPassDescriptor == nil)
    {
        [commandBuffer commit];
        return;
    }

    // *** PASS 2: Render ImGui UI on top ***
    ImGui_ImplMetal_NewFrame(renderPassDescriptor);
#if TARGET_OS_OSX
    ImGui_ImplOSX_NewFrame(view);
#endif
    ImGui::NewFrame();

    // ImGui UI code...
    static bool show_demo_window = true;
    if (show_demo_window)
        ImGui::ShowDemoWindow(&show_demo_window);

    // Rendering
    ImGui::Render();
    ImDrawData* draw_data = ImGui::GetDrawData();

    if (io.ConfigFlags & ImGuiConfigFlags_ViewportsEnable) {
        ImGui::UpdatePlatformWindows();
        ImGui::RenderPlatformWindowsDefault();
    }

    // Don't clear - preserve game viewport rendering
    renderPassDescriptor.colorAttachments[0].loadAction = MTLLoadActionLoad;
    
    id <MTLRenderCommandEncoder> renderEncoder = [commandBuffer renderCommandEncoderWithDescriptor:renderPassDescriptor];
    [renderEncoder pushDebugGroup:@"Dear ImGui rendering"];
    ImGui_ImplMetal_RenderDrawData(draw_data, commandBuffer, renderEncoder);
    [renderEncoder popDebugGroup];
    [renderEncoder endEncoding];

    // Present
    [commandBuffer presentDrawable:view.currentDrawable];
    [commandBuffer commit];
}
```

#### Option C: Modify Rust Renderer (Most Flexible)

Change the Rust renderer to accept a command buffer and encoder instead of creating its own:

```rust
// In metal.rs
pub fn draw(&self, encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>) {
    encoder.setRenderPipelineState(&self.pipeline_state);
    unsafe {
        encoder.setVertexBuffer_offset_atIndex(Some(&*self.vertex_buffer), 0, 0);
        encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::Triangle, 0, 3);
    }
}
```

```c
// In luandabridge.h
void luanda_renderer_draw_with_encoder(void* renderer, id<MTLRenderCommandEncoder> encoder);
```

```objc
// In osx_main.mm
id <MTLRenderCommandEncoder> renderEncoder = [commandBuffer renderCommandEncoderWithDescriptor:renderPassDescriptor];

// Render game first
luanda_renderer_draw_with_encoder(self.rustRenderer, renderEncoder);

// Then render ImGui
[renderEncoder pushDebugGroup:@"Dear ImGui rendering"];
ImGui_ImplMetal_RenderDrawData(draw_data, commandBuffer, renderEncoder);
[renderEncoder popDebugGroup];
[renderEncoder endEncoding];
```

### 6. Update Build Configuration

Add the bridge header include directory to `src/editor/imgui/CMakeLists.txt`:

```cmake
# After add_executable(LuandaEditor...)
target_include_directories(LuandaEditor PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../swiftui/include
)
```

## Bridge Interface Reference

From `src/editor/swiftui/include/luandabridge.h`:

```c
void* luanda_renderer_create(id<MTLDevice> device);
void luanda_renderer_draw(void* renderer, MTLRenderPassDescriptor* descriptor, id<MTLDrawable> drawable);
void luanda_renderer_destroy(void* renderer);
```

## Key Differences from SwiftUI

| Aspect | SwiftUI | ImGui |
|--------|---------|-------|
| **View Structure** | Separate `MetalView` component | Single MTKView for everything |
| **Rendering Layers** | Metal view as detail view | Game background + ImGui overlay |
| **Coordinator** | `MetalViewCoordinator` class | `AppViewController` serves this role |
| **Lifecycle** | Managed by SwiftUI | Manual in Objective-C++ |

## Result

After integration:
- ✅ Full-screen game viewport rendered by Rust Metal renderer
- ✅ ImGui UI rendered as transparent overlay on top
- ✅ Same rendering backend used in both SwiftUI and ImGui editors
- ✅ Consistent Metal rendering pipeline

## Troubleshooting

### "Each CAMetalLayerDrawable can only be presented once!" + SIGSEGV
**Cause**: The Rust renderer creates its own command buffer and tries to present the drawable, then ImGui also tries to present the same drawable.

**Solution**: Use Option C above - modify the Rust renderer to accept an encoder instead of creating its own command buffer. The ImGui code should manage the command buffer and presentation.

### Renderer not visible
- Ensure rendering happens in the correct order
- Check that ImGui's clear color has proper alpha for transparency
- Verify the load action is set correctly (MTLLoadActionLoad to preserve previous rendering)

### Crash on startup
- Verify bridge header path is correct in CMakeLists.txt
- Ensure LuandaEngine is properly linked
- Check that renderer is properly initialized before first draw call

### Render pass issues
- Only ONE command buffer should present the drawable
- If using shared rendering, use MTLLoadActionLoad instead of MTLLoadActionClear
- Don't create multiple render command encoders for the same drawable in the same frame
