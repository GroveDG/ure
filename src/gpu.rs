use std::mem::MaybeUninit;

use wgpu::{
    CommandEncoder, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceTexture, wgt::SurfaceConfiguration,
};

use crate::declare_components;

pub type Surface = wgpu::Surface<'static>;

const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

declare_components!{
    window: crate::app::Window,
    surface: crate::gpu::Surface,
}

pub fn init_surfaces(
    windows: &[crate::app::Window],
    surfaces: &mut [MaybeUninit<Surface>],
    gpu: &Gpu,
) {
    for (w, s) in windows.iter().zip(surfaces.iter_mut()) {
        let size = w.inner_size();
        let surface = gpu.instance.create_surface(w.clone()).unwrap();
        surface.configure(
            &gpu.device,
            &SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: SURFACE_FORMAT,
                view_formats: vec![],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: size.width,
                height: size.height,
                desired_maximum_frame_latency: 2,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );
        s.write(surface);
    }
}

pub fn init_encoders(amount: usize, gpu: &Gpu) -> Vec<CommandEncoder> {
    let mut encoders = Vec::with_capacity(amount);
    for _i in 0..amount {
        encoders.push(gpu.device.create_command_encoder(&Default::default()))
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

pub fn clear_surfaces(
    encoders: &mut [CommandEncoder],
    surface_textures: &[SurfaceTexture],
    color: wgpu::Color,
) {
    for (surface_texture, encoder) in surface_textures.iter().zip(encoders) {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        format: Some(SURFACE_FORMAT),
                        ..Default::default()
                    }),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}

pub fn submit_encoders(encoders: Vec<wgpu::CommandEncoder>, gpu: &Gpu) {
    gpu.queue
        .submit(encoders.into_iter().map(|encoder| encoder.finish()));
}

pub fn present_surfaces(surface_textures: Vec<SurfaceTexture>) {
    for surface_texture in surface_textures {
        surface_texture.present();
    }
}

pub struct Gpu {
    pub instance: Instance,
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
            device,
            queue,
        }
    }
}
