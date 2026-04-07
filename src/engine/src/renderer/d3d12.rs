use std::ffi::c_void;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D;
use windows::Win32::Graphics::{Direct3D::Dxc::*, Direct3D::*, Direct3D12::*, Dxgi::Common::*};
use windows::core::{Interface, GUID, HRESULT, PCSTR, PCWSTR, PWSTR};

use crate::renderer::{
    LuandaBackend, LuandaExternalDevice, LuandaTextureHandle, Renderer, TextureHandle,
};

pub struct D3D12Renderer {
    device: ID3D12Device,
    command_queue: ID3D12CommandQueue,
    command_allocator: ID3D12CommandAllocator,
    command_list: ID3D12GraphicsCommandList,
    rtv_heap: ID3D12DescriptorHeap,
    pipeline_state: ID3D12PipelineState,
    vertex_buffer: ID3D12Resource,
    render_texture: Option<ID3D12Resource>,
    texture_width: usize,
    texture_height: usize,
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

        let pipeline_state = Self::create_pipeline(&device);

        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                Some(&pipeline_state),
            )
        }
        .expect("Failed to create command list");
        unsafe {
            command_list.Close().expect("Failed to close command list");
        }

        // Create vertex buffer (triangle vertices)
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0, // Top
            -0.5, -0.5, 0.0, // Bottom left
            0.5, -0.5, 0.0, // Bottom right
        ];
        let mut vertex_buffer: Option<ID3D12Resource> = None;
        unsafe {
            device.CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_UPLOAD,
                    CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                    MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                    CreationNodeMask: 0,
                    VisibleNodeMask: 0,
                },
                D3D12_HEAP_FLAG_NONE,
                &D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                    Alignment: 0,
                    Width: (vertices.len() * std::mem::size_of::<f32>()) as u64,
                    Height: 1,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_UNKNOWN,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                    Flags: D3D12_RESOURCE_FLAG_NONE,
                },
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                &mut vertex_buffer,
            )
        }
        .expect("Failed to create vertex buffer");
        // Copy vertex data to buffer
        let mut vertex_data_ptr: *mut f32 = std::ptr::null_mut();
        unsafe {
            vertex_buffer
                .as_ref()
                .unwrap()
                .Map(0, None, Some(&mut vertex_data_ptr as *mut _ as *mut _))
                .expect("Failed to map vertex buffer");
        };
        unsafe {
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), vertex_data_ptr, vertices.len());
            vertex_buffer.as_ref().unwrap().Unmap(0, None);
        }

        Self {
            device,
            command_queue,
            command_allocator,
            command_list,
            rtv_heap,
            pipeline_state,
            vertex_buffer: vertex_buffer.as_ref().unwrap().clone(),
            render_texture: None,
            texture_width: 0,
            texture_height: 0,
        }
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

        Ok(())
    }

    fn create_shader_library(vs: &str, ps: Option<&str>) -> anyhow::Result<(Option<ID3DBlob>, Option<ID3DBlob>)> {
        let compiler: IDxcCompiler3 = unsafe { DxcCreateInstance(&CLSID_DxcCompiler).unwrap() };
        
        let vs_wide: Vec<u16> = vs.encode_utf16().chain(Some(0)).collect();
        let vs_blob= unsafe {
            compiler.Compile(
                &DxcBuffer {
                    Ptr: vs_wide.as_ptr() as *const _,
                    Size: vs_wide.len() * 2,
                    Encoding: 0,
                },
                Some(&[
                    PCWSTR::from_raw(b"-E\0".as_ptr() as *const u16),
                    PCWSTR::from_raw(b"VSMain\0".as_ptr() as *const u16),
                    PCWSTR::from_raw(b"-T\0".as_ptr() as *const u16),
                    PCWSTR::from_raw(b"vs_6_0\0".as_ptr() as *const u16),
                ]),
                None,
            )?
        };
        
        let ps_blob = if let Some(ps) = ps {
            let ps_wide: Vec<u16> = ps.encode_utf16().chain(Some(0)).collect();
            Some(unsafe {
                compiler.Compile(
                    &DxcBuffer {
                        Ptr: ps_wide.as_ptr() as *const _,
                        Size: ps_wide.len() * 2,
                        Encoding: 0,
                    },
                    Some(&[
                        PCWSTR::from_raw(b"-E\0".as_ptr() as *const u16),
                        PCWSTR::from_raw(b"PSMain\0".as_ptr() as *const u16),
                        PCWSTR::from_raw(b"-T\0".as_ptr() as *const u16),
                        PCWSTR::from_raw(b"ps_6_0\0".as_ptr() as *const u16),
                    ]),
                    None,
                )?
            })
        } else {
            None
        };
        
        Ok((Some(vs_blob), ps_blob))
    }

    fn create_pipeline(device: &ID3D12Device) -> ID3D12PipelineState {
        let mut pipeline_state_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC::default();
        pipeline_state_desc.RasterizerState = D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: true.into(),
            DepthBias: 0 as i32,
            DepthBiasClamp: 0 as f32,
            SlopeScaledDepthBias: 0 as f32,
            DepthClipEnable: false.into(),
            MultisampleEnable: false.into(),
            AntialiasedLineEnable: true.into(),
            ForcedSampleCount: 0 as u32,
            ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        };

        let (vs, _ps) = Self::create_shader_library(
            r#"
            struct PSInput {
                float4 position : SV_POSITION;
            };

            PSInput VSMain(uint vertexId : SV_VertexID) {
                float3 vertices[3] = {
                    float3(0.0, 0.5, 0.0),   // Top
                    float3(-0.5, -0.5, 0.0), // Bottom left
                    float3(0.5, -0.5, 0.0)   // Bottom right
                };
                PSInput output;
                output.position = float4(vertices[vertexId], 1.0);
                return output;
            }
            "#,
            None,
        )        .expect("Failed to compile vertex shader");

        pipeline_state_desc.VS = vs.as_ref().map(|blob| D3D12_SHADER_BYTECODE {
            pShaderBytecode: unsafe { blob.GetBufferPointer() } as *const _,
            BytecodeLength: unsafe { blob.GetBufferSize() } as usize,
        }).unwrap();

        let pipeline_state = unsafe { device.CreateGraphicsPipelineState(&pipeline_state_desc) }
            .expect("Failed to create pipeline state");

        pipeline_state
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
        let vbv = D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: unsafe { self.vertex_buffer.GetGPUVirtualAddress() },
            SizeInBytes: (9 * std::mem::size_of::<f32>()) as u32,
            StrideInBytes: (3 * std::mem::size_of::<f32>()) as u32,
        };

        unsafe {
            self.command_allocator.Reset()?;
            self.command_list
                .Reset(&self.command_allocator, Some(&self.pipeline_state))?;

            let rtv_handle = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            self.command_list
                .OMSetRenderTargets(1, Some(&rtv_handle), false, None);
            self.command_list
                .ClearRenderTargetView(rtv_handle, &clear_color, None);

            self.command_list.RSSetViewports(&[viewport]);
            self.command_list.RSSetScissorRects(&[scissor]);
            self.command_list
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.command_list.IASetVertexBuffers(0, Some(&[vbv]));
            self.command_list.DrawInstanced(3, 1, 0, 0);
            self.command_list.Close()?;

            let command_list: ID3D12CommandList = self.command_list.cast()?;
            self.command_queue
                .ExecuteCommandLists(&[Some(command_list)]);

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
