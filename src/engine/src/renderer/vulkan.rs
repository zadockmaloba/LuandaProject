use std::ffi::c_void;
use std::sync::Arc;

use vulkano::{
    VulkanLibrary, VulkanObject,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents, SubpassEndInfo,
    },
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags},
    device::physical::PhysicalDeviceType,
    format::Format,
    image::{Image, ImageType, ImageUsage},
    image::sys::ImageCreateInfo,
    image::view::ImageView,
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexInputState,
            viewport::{Viewport, ViewportState},
        },
    },
    pipeline::layout::PipelineLayoutCreateInfo,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::{self, GpuFuture},
};

use crate::renderer::{
    LuandaBackend, LuandaExternalDevice, LuandaTextureHandle, Renderer, TextureHandle,
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) out vec4 frag_color;

            void main() {
                vec2 positions[3] = vec2[](
                    vec2( 0.0,  0.6),
                    vec2( 0.6, -0.6),
                    vec2(-0.6, -0.6)
                );
                vec3 colors[3] = vec3[](
                    vec3(1.0,  0.25, 0.2),
                    vec3(0.2,  1.0,  0.35),
                    vec3(0.2,  0.45, 1.0)
                );
                gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
                frag_color  = vec4(colors[gl_VertexIndex], 1.0);
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in  vec4 frag_color;
            layout(location = 0) out vec4 out_color;

            void main() {
                out_color = frag_color;
            }
        ",
    }
}

const RENDER_FORMAT: Format = Format::R8G8B8A8_UNORM;

pub struct VulkanRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    render_texture: Option<Arc<Image>>,
    framebuffer: Option<Arc<Framebuffer>>,
    texture_width: u32,
    texture_height: u32,
}

impl VulkanRenderer {
    pub fn new() -> anyhow::Result<Self> {
        let library = VulkanLibrary::new()
            .map_err(|e| anyhow::anyhow!("Failed to load Vulkan library: {e}"))?;

        let instance = Instance::new(library, InstanceCreateInfo::default())
            .map_err(|e| anyhow::anyhow!("Failed to create Vulkan instance: {e}"))?;

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .map_err(|e| anyhow::anyhow!("Failed to enumerate physical devices: {e}"))?
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(_, q)| q.queue_flags.intersects(QueueFlags::GRAPHICS))
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .ok_or_else(|| anyhow::anyhow!("No suitable Vulkan physical device found"))?;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: DeviceExtensions::empty(),
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Vulkan device: {e}"))?;

        let queue = queues.next().expect("At least one queue was requested");

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: RENDER_FORMAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create render pass: {e}"))?;

        let pipeline = Self::create_pipeline(device.clone(), render_pass.clone())?;

        Ok(Self {
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            render_pass,
            pipeline,
            render_texture: None,
            framebuffer: None,
            texture_width: 0,
            texture_height: 0,
        })
    }

    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
    ) -> anyhow::Result<Arc<GraphicsPipeline>> {
        let vs = vs::load(device.clone())
            .map_err(|e| anyhow::anyhow!("Failed to load vertex shader: {e}"))?
            .entry_point("main")
            .ok_or_else(|| anyhow::anyhow!("No 'main' entry point in vertex shader"))?;
        let fs = fs::load(device.clone())
            .map_err(|e| anyhow::anyhow!("Failed to load fragment shader: {e}"))?
            .entry_point("main")
            .ok_or_else(|| anyhow::anyhow!("No 'main' entry point in fragment shader"))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(device.clone(), PipelineLayoutCreateInfo::default())
            .map_err(|e| anyhow::anyhow!("Failed to create pipeline layout: {e}"))?;

        let subpass = Subpass::from(render_pass, 0)
            .ok_or_else(|| anyhow::anyhow!("Subpass 0 not found in render pass"))?;

        GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(VertexInputState::default()),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    1,
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create graphics pipeline: {e}"))
    }

    fn ensure_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        if self.render_texture.is_some()
            && self.texture_width == width
            && self.texture_height == height
        {
            return Ok(());
        }

        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: RENDER_FORMAT,
                extent: [width, height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create render texture: {e}"))?;

        let view = ImageView::new_default(image.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create image view: {e}"))?;

        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("Failed to create framebuffer: {e}"))?;

        self.render_texture = Some(image);
        self.framebuffer = Some(framebuffer);
        self.texture_width = width;
        self.texture_height = height;

        Ok(())
    }
}

impl Renderer for VulkanRenderer {
    fn render_to_texture(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.ensure_texture(width, height)?;

        let framebuffer = self
            .framebuffer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Framebuffer was not created"))?
            .clone();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [width as f32, height as f32],
            depth_range: 0.0..=1.0,
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create command buffer builder: {e}"))?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.1_f32, 0.1, 0.15, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow::anyhow!("begin_render_pass failed: {e}"))?
            .set_viewport(0, [viewport].into_iter().collect())
            .map_err(|e| anyhow::anyhow!("set_viewport failed: {e}"))?
            .bind_pipeline_graphics(self.pipeline.clone())
            .map_err(|e| anyhow::anyhow!("bind_pipeline_graphics failed: {e}"))?;

        unsafe { builder.draw(3, 1, 0, 0) }
            .map_err(|e| anyhow::anyhow!("draw failed: {e}"))?;

        builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| anyhow::anyhow!("end_render_pass failed: {e}"))?;

        let command_buffer = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build command buffer: {e}"))?;

        // Submit and wait so the texture is ready when the editor samples it.
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| anyhow::anyhow!("then_execute failed: {e}"))?
            .then_signal_fence_and_flush()
            .map_err(|e| anyhow::anyhow!("then_signal_fence_and_flush failed: {e}"))?;

        future
            .wait(None)
            .map_err(|e| anyhow::anyhow!("GPU wait failed: {e}"))?;

        Ok(())
    }

    fn get_texture_handle(&self) -> Option<TextureHandle> {
        self.render_texture.as_ref().map(|img| {
            // VkImage is a non-dispatchable u64 handle; cast to *mut c_void for the C API.
            let raw = img.handle().as_raw() as usize as *mut c_void;
            TextureHandle::Vulkan(raw)
        })
    }
}

// ── C FFI ────────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_create(
    external_device: *mut LuandaExternalDevice,
) -> *mut c_void {
    if external_device.is_null() {
        return std::ptr::null_mut();
    }

    let external_device = unsafe { &*external_device };
    if external_device.backend != LuandaBackend::Vulkan {
        return std::ptr::null_mut();
    }

    let creation_result = std::panic::catch_unwind(|| {
        match VulkanRenderer::new() {
            Ok(renderer) => Box::into_raw(Box::new(renderer)) as *mut c_void,
            Err(_) => std::ptr::null_mut(),
        }
    });

    creation_result.unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_draw(renderer: *mut c_void, width: u32, height: u32) {
    if renderer.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| {
        let renderer = unsafe { &mut *(renderer as *mut VulkanRenderer) };
        let _ = renderer.render_to_texture(width, height);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_get_texture(
    renderer: *mut c_void,
    out_texture: *mut LuandaTextureHandle,
) -> i32 {
    if renderer.is_null() || out_texture.is_null() {
        return 0;
    }

    let result = std::panic::catch_unwind(|| {
        let renderer = unsafe { &*(renderer as *const VulkanRenderer) };
        match renderer.get_texture_handle() {
            Some(TextureHandle::Vulkan(handle)) if !handle.is_null() => {
                unsafe {
                    (*out_texture).backend = LuandaBackend::Vulkan;
                    (*out_texture).handle = handle;
                }
                1
            }
            _ => 0,
        }
    });

    result.unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn luanda_renderer_destroy(renderer: *mut c_void) {
    if renderer.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| {
        unsafe { drop(Box::from_raw(renderer as *mut VulkanRenderer)) };
    });
}
