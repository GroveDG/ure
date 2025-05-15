use std::sync::Arc;

use wgpu::{Device, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::window::Window;

use crate::sys::{Components, UID};

use super::GPU;

#[derive(Debug, Default)]
pub struct Surfaces<'a> {
    surfaces: Components<WindowSurface<'a>>,
}

#[derive(Debug)]
pub struct WindowSurface<'a> {
    pub window: Arc<Window>,
    pub surface: Surface<'a>,
    pub capabilities: SurfaceCapabilities,
}

impl<'a> Surfaces<'a> {
    pub fn insert(&mut self, uid: UID, window: Arc<Window>, gpu: &GPU) {
        let surface = gpu.instance.create_surface(Arc::clone(&window)).unwrap();
        let capabilities = surface.get_capabilities(&gpu.adapter);
        self.surfaces.insert(
            uid,
            WindowSurface {
                window,
                surface,
                capabilities,
            },
        );
    }
    pub fn configure(&self, uid: &UID, device: &Device) {
        let Some(surface) = self.get(uid) else {
            return;
        };
        let size = surface.window.inner_size();
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.capabilities.formats[0],
            view_formats: vec![surface.capabilities.formats[0].add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.surface.configure(device, &config);
    }
    pub fn get(&self, uid: &UID) -> Option<&WindowSurface<'a>> {
        self.surfaces.get(uid)
    }
    pub fn get_mut(&mut self, uid: &UID) -> Option<&mut WindowSurface<'a>> {
        self.surfaces.get_mut(uid)
    }
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut WindowSurface<'a>> {
        self.surfaces.values_mut()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&UID, &mut WindowSurface<'a>)> {
        self.surfaces.iter_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&UID, &WindowSurface<'a>)> {
        self.surfaces.iter()
    }
    pub fn values(&self) -> impl Iterator<Item = &WindowSurface<'a>> {
        self.surfaces.values()
    }
}
