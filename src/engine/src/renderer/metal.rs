use core::ffi::c_void;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{ns_string, NSString};
use objc2_metal::*;
use std::ptr::NonNull;

pub struct MetalRenderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
}

impl MetalRenderer {
    pub fn new(device: Retained<ProtocolObject<dyn MTLDevice>>) -> Self {
        let command_queue = device.newCommandQueue()
            .expect("Failed to create command queue");
        
        // Create shader library
        let library = Self::create_shader_library(&device);
        
        // Create render pipeline
        let pipeline_state = Self::create_pipeline(&device, &library);
        
        // Create vertex buffer (example: triangle)
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,   // Top
           -0.5, -0.5, 0.0,  // Bottom left
            0.5, -0.5, 0.0,  // Bottom right
        ];
        let vertex_ptr = NonNull::new(vertices.as_ptr() as *mut c_void)
            .expect("Vertex pointer should not be null");
        let vertex_buffer = unsafe {
            device
                .newBufferWithBytes_length_options(
                    vertex_ptr,
                    std::mem::size_of_val(&vertices),
                    MTLResourceOptions::StorageModeShared,
                )
        }
        .expect("Failed to create vertex buffer");
        
        Self {
            device,
            command_queue,
            pipeline_state,
            vertex_buffer,
        }
    }
    
    pub fn draw(&self, descriptor: &MTLRenderPassDescriptor) {
        let command_buffer = self.command_queue.commandBuffer()
            .expect("Failed to create command buffer");
        
        let encoder = command_buffer.renderCommandEncoderWithDescriptor(descriptor)
            .expect("Failed to create encoder");
        
        encoder.setRenderPipelineState(&self.pipeline_state);
        unsafe {
            encoder.setVertexBuffer_offset_atIndex(Some(&*self.vertex_buffer), 0, 0);
        }

        unsafe {
            encoder.drawPrimitives_vertexStart_vertexCount(
                MTLPrimitiveType::Triangle,
                0,
                3
            );
        }
        
        encoder.endEncoding();
        command_buffer.commit();
    }
    
    fn create_shader_library(
        device: &ProtocolObject<dyn MTLDevice>,
    ) -> Retained<ProtocolObject<dyn MTLLibrary>> {
        let shader_source = r#"
            #include <metal_stdlib>
            using namespace metal;
            
            struct VertexOut {
                float4 position [[position]];
                float4 color;
            };
            
            vertex VertexOut vertex_main(uint vertexID [[vertex_id]],
                                         constant float3* vertices [[buffer(0)]]) {
                VertexOut out;
                out.position = float4(vertices[vertexID], 1.0);
                // Rainbow colors
                float3 colors[3] = {
                    float3(1.0, 0.0, 0.0),
                    float3(0.0, 1.0, 0.0),
                    float3(0.0, 0.0, 1.0)
                };
                out.color = float4(colors[vertexID], 1.0);
                return out;
            }
            
            fragment float4 fragment_main(VertexOut in [[stage_in]]) {
                return in.color;
            }
        "#;

        let shader_source_ns = NSString::from_str(shader_source);
        device
            .newLibraryWithSource_options_error(&shader_source_ns, None)
            .expect("Failed to create shader library")
    }
    
    fn create_pipeline(
        device: &ProtocolObject<dyn MTLDevice>,
        library: &ProtocolObject<dyn MTLLibrary>,
    ) -> Retained<ProtocolObject<dyn MTLRenderPipelineState>> {
        let descriptor = MTLRenderPipelineDescriptor::new();
        
        let vertex_fn = library
            .newFunctionWithName(ns_string!("vertex_main"))
            .expect("Failed to find vertex function");
        let fragment_fn = library
            .newFunctionWithName(ns_string!("fragment_main"))
            .expect("Failed to find fragment function");
        
        descriptor.setVertexFunction(Some(&vertex_fn));
        descriptor.setFragmentFunction(Some(&fragment_fn));
        
        unsafe {
            let color_attachment = descriptor.colorAttachments()
                .objectAtIndexedSubscript(0);
            color_attachment.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        }
        
        device
            .newRenderPipelineStateWithDescriptor_error(&descriptor)
            .expect("Failed to create pipeline")
    }
}

// C FFI exports
#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_create(
    device: *mut ProtocolObject<dyn MTLDevice>,
) -> *mut MetalRenderer {
    let retained_device = unsafe {
        let device = NonNull::new(device).expect("Null device");
        Retained::retain(device.as_ptr()).expect("Failed to retain device")
    };
    println!("Creating MetalRenderer");
    Box::into_raw(Box::new(MetalRenderer::new(retained_device)))
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_draw(
    renderer: *mut MetalRenderer,
    descriptor: *const MTLRenderPassDescriptor,
) {
    let renderer = unsafe { &*renderer };
    let descriptor = unsafe { &*descriptor };
    renderer.draw(descriptor);
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_destroy(renderer: *mut MetalRenderer) {
    println!("Destroying MetalRenderer");
    unsafe { drop(Box::from_raw(renderer)) };
}