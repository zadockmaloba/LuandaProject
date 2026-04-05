# Renderer Interface Abstraction (Metal, D3D12, Vulkan)

## Goal

Define a single renderer interface that can be implemented by Metal, D3D12, and Vulkan backends, while keeping backend-specific native types isolated.

Current state:
- The active implementation is Metal-specific in `src/renderer/metal.rs`.
- The C exports are Metal-only (`id<MTLTexture>` in `include/renderer_metal.h`).
- `src/renderer/mod.rs` only conditionally exposes backend modules and has no shared abstraction.

## Design Summary

Use two layers:
1. A Rust trait for shared rendering behavior.
2. Backend-tagged opaque handle enums for native device/texture interop.

This keeps trait objects practical (`Box<dyn Renderer>`) and avoids tying common APIs to one graphics API.

## Core Rust Types

Add these to `src/renderer/mod.rs` (or split into a `src/renderer/api.rs` module).

```rust
pub enum Backend {
    Metal,
    D3D12,
    Vulkan,
}

pub enum ExternalDevice {
    // Opaque native pointers from host app
    Metal(*mut core::ffi::c_void),   // id<MTLDevice>
    D3D12(*mut core::ffi::c_void),   // ID3D12Device*
    Vulkan(*mut core::ffi::c_void),  // VkDevice or backend context pointer
}

pub enum TextureHandle {
    Metal(*mut core::ffi::c_void),   // id<MTLTexture>
    D3D12(*mut core::ffi::c_void),   // ID3D12Resource*
    Vulkan(*mut core::ffi::c_void),  // VkImage / image view / wrapper pointer
}

pub trait Renderer {
    fn render_to_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()>;
    fn texture_handle(&self) -> Option<TextureHandle>;
}
```

### Why use enums + opaque pointers?

- Allows one trait object for all backends.
- Keeps platform-specific types out of the cross-platform trait.
- Makes C ABI simpler and portable.

## Backend Implementations

Implement `Renderer` for each backend struct:
- `MetalRenderer` in `src/renderer/metal.rs`
- `D3D12Renderer` in `src/renderer/d3d12.rs`
- `VulkanRenderer` in `src/renderer/vulkan.rs` (new)

Example implementation direction for Metal:

```rust
impl Renderer for MetalRenderer {
    fn render_to_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.render_to_texture(width as usize, height as usize);
        Ok(())
    }

    fn texture_handle(&self) -> Option<TextureHandle> {
        self.get_texture()
            .map(|t| TextureHandle::Metal(t as *mut core::ffi::c_void))
    }
}
```

## Renderer Factory

Create a backend/device validated factory:

```rust
pub fn create_renderer(
    backend: Backend,
    device: ExternalDevice,
) -> anyhow::Result<Box<dyn Renderer>> {
    match (backend, device) {
        (Backend::Metal, ExternalDevice::Metal(dev)) => {
            // Build MetalRenderer from native pointer
            // return Ok(Box::new(renderer));
            todo!()
        }
        (Backend::D3D12, ExternalDevice::D3D12(dev)) => {
            // Build D3D12Renderer
            todo!()
        }
        (Backend::Vulkan, ExternalDevice::Vulkan(dev)) => {
            // Build VulkanRenderer
            todo!()
        }
        _ => anyhow::bail!("backend/device mismatch"),
    }
}
```

## C ABI Migration Plan

Current C exports are backend-specific (Metal names and types).

Introduce backend-agnostic exports:

```c
typedef enum LuandaBackend {
    LUANDA_BACKEND_METAL = 0,
    LUANDA_BACKEND_D3D12 = 1,
    LUANDA_BACKEND_VULKAN = 2,
} LuandaBackend;

typedef struct LuandaTextureHandle {
    LuandaBackend backend;
    void* handle;
} LuandaTextureHandle;

void* luanda_renderer_create(int backend, void* device);
void  luanda_renderer_render(void* renderer, uint32_t width, uint32_t height);
int   luanda_renderer_get_texture(void* renderer, LuandaTextureHandle* out_texture);
void  luanda_renderer_destroy(void* renderer);
```

Notes:
- `device` and `handle` are opaque to C and interpreted by backend tag.
- Return status codes (`int`) for FFI robustness.
- Keep old Metal exports during transition, then deprecate.

## Suggested File Layout

```text
src/renderer/
  mod.rs           // shared API, trait, enums, factory
  metal.rs         // Metal backend + impl Renderer
  d3d12.rs         // D3D12 backend + impl Renderer
  vulkan.rs        // Vulkan backend + impl Renderer
include/
  renderer.h       // backend-agnostic C API
  renderer_metal.h // optional compatibility header (temporary)
```

## Incremental Implementation Steps

1. Add shared trait/enums to `src/renderer/mod.rs`.
2. Implement `Renderer` for Metal without changing behavior.
3. Add factory and convert create path to `Box<dyn Renderer>`.
4. Add backend-agnostic C header (`include/renderer.h`).
5. Keep Metal-specific header/API as compatibility shim.
6. Add D3D12 implementation to satisfy trait.
7. Add Vulkan implementation to satisfy trait.
8. Remove/deprecate backend-specific exports once all consumers migrate.

## Practical Constraints and Decisions

- Avoid associated types in trait if dynamic dispatch is required.
- Keep pointers opaque in FFI to avoid ABI coupling.
- Convert `usize` to fixed-width `u32` at ABI boundary for consistency.
- Validate backend-device pairing at creation time.

## Future Extension Points

If needed later, extend trait with:

```rust
fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()>;
fn submit_scene(&mut self, scene: &SceneGraph) -> anyhow::Result<()>;
fn wait_idle(&mut self) -> anyhow::Result<()>;
```

Only add methods that every backend can support cleanly.
