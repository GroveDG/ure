use std::mem::MaybeUninit;

use color::Srgb;
use wgpu::{
    Adapter, CommandEncoder, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceTexture,
};
use winit::dpi::PhysicalSize;

use crate::app::SpanMask;

#[cfg(feature = "2d")]
pub mod two;

pub type Surface = wgpu::Surface<'static>;

const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub static GPU: std::sync::LazyLock<Gpu> =
    std::sync::LazyLock::new(|| futures::executor::block_on(Gpu::new()));

pub type Color = color::AlphaColor<Srgb>;

pub fn init_surfaces<'a>(
    windows: &[crate::app::Window],
    surfaces: &'a mut [MaybeUninit<Surface>],
) -> &'a mut [Surface] {
    for i in 0..windows.len() {
        let window = windows[i].clone();
        let size = window.inner_size();
        let surface = GPU.instance.create_surface(window).unwrap();
        surface.configure(
            &GPU.device,
            &surface
                .get_default_config(&GPU.adapter, size.width, size.height)
                .unwrap(),
        );
        surfaces[i].write(surface);
    }
    unsafe { std::mem::transmute(surfaces) }
}
pub fn init_window_sizes<'a>(
    windows: &[crate::app::Window],
    sizes: &'a mut [MaybeUninit<PhysicalSize<u32>>],
) -> &'a mut [PhysicalSize<u32>] {
    for i in 0..windows.len() {
        sizes[i].write(windows[i].inner_size());
    }
    unsafe { std::mem::transmute(sizes) }
}

pub fn init_windows_and_surfaces(
    span: &mut crate::app::SpanInit,
    event_loop: &winit::event_loop::ActiveEventLoop,
) {
    let window = crate::app::init_windows(span.window.take().unwrap(), event_loop);
    init_surfaces(window, span.surface.take().unwrap());
    init_window_sizes(window, span.window_size.take().unwrap());
}

pub const MASK: SpanMask = SpanMask {
    window: true,
    surface: true,
    window_size: true,
    ..SpanMask::NONE
};

pub fn reconfigure_surfaces(
    surfaces: &[Surface],
    windows: &[crate::app::Window],
    sizes: &mut [PhysicalSize<u32>],
) {
    for i in 0..surfaces.len() {
        let size = windows[i].inner_size();
        if size != sizes[i] {
            sizes[i] = size;
            surfaces[i].configure(
                &GPU.device,
                &surfaces[i]
                    .get_default_config(&GPU.adapter, size.width, size.height)
                    .unwrap(),
            );
        }
    }
}

pub fn init_encoders(amount: usize) -> Vec<CommandEncoder> {
    let mut encoders = Vec::with_capacity(amount);
    for _ in 0..amount {
        encoders.push(GPU.device.create_command_encoder(&Default::default()))
    }
    encoders
}

pub fn init_surface_textures(surfaces: &[Surface]) -> Vec<wgpu::SurfaceTexture> {
    let mut surface_textures: Vec<wgpu::SurfaceTexture> = Vec::with_capacity(surfaces.len());
    for surface in surfaces {
        surface_textures.push(surface.get_current_texture().unwrap());
    }
    surface_textures
}

pub fn begin_passes<'a>(
    encoders: &'a mut [CommandEncoder],
    surface_textures: &[SurfaceTexture],
    color: wgpu::Color,
) -> Vec<wgpu::RenderPass<'a>> {
    let mut passes = Vec::with_capacity(encoders.len());
    for (i, encoder) in encoders.iter_mut().enumerate() {
        passes.push(encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(
                    wgpu::RenderPassColorAttachment {
                        view: &surface_textures[i].texture.create_view(
                            &wgpu::TextureViewDescriptor {
                                format: Some(SURFACE_FORMAT),
                                ..Default::default()
                            },
                        ),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color),
                            store: wgpu::StoreOp::Store,
                        },
                    },
                ),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }));
    }
    passes
}

pub fn submit_encoders(encoders: Vec<wgpu::CommandEncoder>) {
    GPU.queue
        .submit(encoders.into_iter().map(|encoder| encoder.finish()));
}

pub fn present_surfaces(surface_textures: Vec<SurfaceTexture>) {
    for surface_texture in surface_textures {
        surface_texture.present();
    }
}

pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}
impl Gpu {
    pub async fn new() -> Self {
        let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap();
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
