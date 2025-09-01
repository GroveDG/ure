use std::{mem::MaybeUninit, sync::OnceLock};

use color::Srgb;
use wgpu::{
    Adapter, CommandEncoder, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceTexture, TextureFormat,
};
use winit::dpi::PhysicalSize;

use crate::app::Window;

pub mod bind;
pub mod instancing;
pub mod rendering;
#[cfg(feature = "2d")]
pub mod two;
pub mod vertex;

pub type Surface = wgpu::Surface<'static>;

pub static GPU: std::sync::LazyLock<Gpu> =
    std::sync::LazyLock::new(|| futures::executor::block_on(Gpu::new()));
pub static SURFACE_FORMAT: OnceLock<TextureFormat> = OnceLock::new();

pub type Color = color::AlphaColor<Srgb>;

pub fn init_surfaces(windows: &[Window], surfaces: &mut [MaybeUninit<Surface>]) {
    for i in 0..surfaces.len() {
        let window = windows[i].clone();
        let size = window.inner_size();
        let surface = GPU.instance.create_surface(window).unwrap();
        let config = &surface
            .get_default_config(&GPU.adapter, size.width, size.height)
            .unwrap();
        // Assert that all surfaces have the same texture format.
        // This simplifies everything greatly at the cost of some
        // obscure situations which are not supported.
        assert_eq!(config.format, *SURFACE_FORMAT.get_or_init(|| config.format));
        surface.configure(&GPU.device, config);
        surfaces[i].write(surface);
    }
}
pub fn init_surface_sizes(windows: &[Window], sizes: &mut [MaybeUninit<PhysicalSize<u32>>]) {
    for i in 0..sizes.len() {
        sizes[i].write(windows[i].inner_size());
    }
}

pub fn reconfigure_surfaces(
    windows: &[Window],
    surfaces: &[Surface],
    sizes: &mut [PhysicalSize<u32>],
) {
    for i in 0..surfaces.len() {
        let size = windows[i].inner_size();
        if size != sizes[i] {
            sizes[i] = size;
            let config = &surfaces[i]
                .get_default_config(&GPU.adapter, size.width, size.height)
                .unwrap();
            surfaces[i].configure(&GPU.device, config);
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
                                format: Some(surface_textures[i].texture.format()),
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
