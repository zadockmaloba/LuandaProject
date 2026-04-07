use std::ffi::c_void;
use windows::Win32::Foundation::{CloseHandle, HANDLE, RECT};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::*};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::Interface;

use crate::renderer::{
    LuandaBackend, LuandaExternalDevice, LuandaTextureHandle, Renderer, TextureHandle,
};

pub struct D3D12Renderer {
    device: ID3D12Device,
    command_queue: ID3D12CommandQueue,
    command_allocator: ID3D12CommandAllocator,
    command_list: ID3D12GraphicsCommandList,
    fence: ID3D12Fence,
    fence_value: u64,
    fence_event: HANDLE,
    rtv_heap: ID3D12DescriptorHeap,
    render_texture: Option<ID3D12Resource>,
    texture_width: usize,
    texture_height: usize,
    texture_state: D3D12_RESOURCE_STATES,
}

impl D3D12Renderer {
    pub fn new(device: ID3D12Device) -> Self {
        // Create command queue
        let command_queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };
        let command_queue = unsafe { device.CreateCommandQueue(&command_queue_desc) }
            .expect("Failed to create command queue");

        let command_allocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .expect("Failed to create command allocator");

        let rtv_heap = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: 1,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            })
        }
        .expect("Failed to create RTV descriptor heap");

        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                None,
            )
        }
        .expect("Failed to create command list");
        unsafe {
            command_list.Close().expect("Failed to close command list");
        }

        let fence =
            unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }.expect("Failed to create fence");

        let fence_event = unsafe { CreateEventW(None, false, false, None) }
            .expect("Failed to create fence event");

        Self {
            device,
            command_queue,
            command_allocator,
            command_list,
            fence,
            fence_value: 0,
            fence_event,
            rtv_heap,
            render_texture: None,
            texture_width: 0,
            texture_height: 0,
            texture_state: D3D12_RESOURCE_STATE_RENDER_TARGET,
        }
    }

    fn wait_for_previous_submission(&self) -> anyhow::Result<()> {
        if unsafe { self.fence.GetCompletedValue() } < self.fence_value {
            unsafe {
                self.fence
                    .SetEventOnCompletion(self.fence_value, self.fence_event)?;
                WaitForSingleObject(self.fence_event, u32::MAX);
            }
        }
        Ok(())
    }

    //fn create_shader_library(device: &ID3D12Device) -> ID3D12PipelineLibrary {}

    fn ensure_texture(&mut self, width: usize, height: usize) -> anyhow::Result<()> {
        if self.render_texture.is_some()
            && self.texture_width == width
            && self.texture_height == height
        {
            return Ok(());
        }

        let mut texture: Option<ID3D12Resource> = None;
        let texture_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: width as u64,
            Height: height as u32,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
        };

        let clear_value = D3D12_CLEAR_VALUE {
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                Color: [0.1, 0.1, 0.15, 1.0],
            },
        };

        unsafe {
            self.device.CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_DEFAULT,
                    CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                    MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                    CreationNodeMask: 0,
                    VisibleNodeMask: 0,
                },
                D3D12_HEAP_FLAG_NONE,
                &texture_desc,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear_value),
                &mut texture,
            )?;
        }

        if let Some(texture) = &texture {
            unsafe {
                let rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
                self.device
                    .CreateRenderTargetView(Some(texture), None, rtv_handle);
            }
        }

        self.render_texture = texture;
        self.texture_width = width;
        self.texture_height = height;
        self.texture_state = D3D12_RESOURCE_STATE_RENDER_TARGET;

        Ok(())
    }

}

impl Renderer for D3D12Renderer {
    fn render_to_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.ensure_texture(width as usize, height as usize)?;

        let texture = self
            .render_texture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Render texture was not created"))?;

        let clear_color = [0.1_f32, 0.1_f32, 0.15_f32, 1.0_f32];
        let viewport = D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        let scissor = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };

        unsafe {
            self.wait_for_previous_submission()?;
            self.command_allocator.Reset()?;
            self.command_list.Reset(&self.command_allocator, None)?;

            if self.texture_state != D3D12_RESOURCE_STATE_RENDER_TARGET {
                let to_render_target = D3D12_RESOURCE_BARRIER {
                    Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    Anonymous: D3D12_RESOURCE_BARRIER_0 {
                        Transition: std::mem::ManuallyDrop::new(
                            D3D12_RESOURCE_TRANSITION_BARRIER {
                                pResource: std::mem::ManuallyDrop::new(Some(texture.clone())),
                                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                                StateBefore: self.texture_state,
                                StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                            },
                        ),
                    },
                };
                self.command_list.ResourceBarrier(&[to_render_target]);
                self.texture_state = D3D12_RESOURCE_STATE_RENDER_TARGET;
            }

            let rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            self.command_list
                .OMSetRenderTargets(1, Some(&rtv_handle), false, None);
            self.command_list
                .ClearRenderTargetView(rtv_handle, &clear_color, None);

            self.command_list.RSSetViewports(&[viewport]);
            self.command_list.RSSetScissorRects(&[scissor]);

            let to_sampled = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::ManuallyDrop::new(Some(texture.clone())),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                    }),
                },
            };
            self.command_list.ResourceBarrier(&[to_sampled]);
            self.texture_state = D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE;
            self.command_list.Close()?;

            let command_list: ID3D12CommandList = self.command_list.cast()?;
            self.command_queue
                .ExecuteCommandLists(&[Some(command_list)]);
            self.fence_value = self.fence_value.saturating_add(1);
            self.command_queue.Signal(&self.fence, self.fence_value)?;

            // Keep the state explicit for callers that may sample the texture afterwards.
            let _ = texture;
        }

        Ok(())
    }

    fn get_texture_handle(&self) -> Option<super::TextureHandle> {
        match &self.render_texture {
            Some(tex) => {
                let handle = TextureHandle::D3D12(tex.as_raw());
                Some(handle)
            }
            None => None,
        }
    }
}

impl Drop for D3D12Renderer {
    fn drop(&mut self) {
        let _ = self.wait_for_previous_submission();
        if !self.fence_event.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.fence_event);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_create(
    backend: i32,
    external_device: *mut LuandaExternalDevice,
) -> *mut c_void {
    if external_device.is_null() {
        return std::ptr::null_mut();
    }

    let external_device = unsafe { &*external_device };
    if backend != LuandaBackend::D3D12 as i32 || external_device.backend != LuandaBackend::D3D12 {
        return std::ptr::null_mut();
    }

    if external_device.device.is_null() {
        return std::ptr::null_mut();
    }

    let creation_result = std::panic::catch_unwind(|| {
        let device_ref = unsafe {
            ID3D12Device::from_raw_borrowed(&external_device.device)
                .expect("Null ID3D12Device pointer")
        };
        let renderer = D3D12Renderer::new(device_ref.clone());
        Box::into_raw(Box::new(renderer)) as *mut c_void
    });

    match creation_result {
        Ok(renderer_ptr) => renderer_ptr,
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_draw(renderer: *mut c_void, width: u32, height: u32) {
    if renderer.is_null() {
        return;
    }

    let draw_result = std::panic::catch_unwind(|| {
        let renderer = unsafe { &mut *(renderer as *mut D3D12Renderer) };
        let _ = renderer.render_to_texture(width, height);
    });

    if draw_result.is_err() {
        return;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_get_texture(
    renderer: *mut c_void,
    out_texture: *mut LuandaTextureHandle,
) -> i32 {
    if renderer.is_null() || out_texture.is_null() {
        return 0;
    }

    let texture_result = std::panic::catch_unwind(|| {
        let renderer = unsafe { &*(renderer as *const D3D12Renderer) };
        match renderer.get_texture_handle() {
            Some(TextureHandle::D3D12(handle)) if !handle.is_null() => {
                unsafe {
                    (*out_texture).backend = LuandaBackend::D3D12;
                    (*out_texture).handle = handle;
                }
                1
            }
            _ => 0,
        }
    });

    texture_result.unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_destroy(renderer: *mut c_void) {
    if renderer.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| {
        unsafe { drop(Box::from_raw(renderer as *mut D3D12Renderer)) };
    });
}
