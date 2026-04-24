#[cfg(target_os = "windows")]
pub mod d3d12;

#[cfg(target_os = "macos")]
pub mod metal;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub mod vulkan;

// use crate::scenegraph::SceneGraph;

pub enum Backend {
    Metal,
    D3D12,
    Vulkan,
}

pub enum ExternalDevice {
    Metal(*mut core::ffi::c_void),   // id<MTLDevice>
    D3D12(*mut core::ffi::c_void),   // ID3D12Device*
    Vulkan(*mut core::ffi::c_void),  // VkDevice or backend context pointer
}

pub enum TextureHandle {
    Metal(*mut core::ffi::c_void),   // id<MTLTexture>
    D3D12(*mut core::ffi::c_void),   // ID3D12Resource*
    Vulkan(*mut core::ffi::c_void),  // VkImage / image view / wrapper pointer
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LuandaBackend {
    Metal = 0,
    D3D12 = 1,
    Vulkan = 2,
}

#[repr(C)]
pub struct LuandaTextureHandle {
    backend: LuandaBackend,
    handle: *mut std::ffi::c_void,
}

#[repr(C)]
pub struct LuandaExternalDevice {
    backend: LuandaBackend,
    device: *mut std::ffi::c_void,
}

pub trait Renderer {
    fn render_to_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()>;
    fn get_texture_handle(&self) -> Option<TextureHandle>;
    // TODO:
    // fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()>;
    // fn submit_scene(&mut self, scene: &SceneGraph) -> anyhow::Result<()>;
    // fn wait_idle(&mut self) -> anyhow::Result<()>;
}