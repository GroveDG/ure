use std::sync::Arc;

use wgpu::{Device, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

use crate::sys::{Components, UID, window::Windows};

use super::GPU;

#[derive(Debug, Default)]
pub struct Surfaces<'a> {
    surfaces: Components<WindowSurface<'a>>,
}

#[derive(Debug)]
pub struct WindowSurface<'a> {
    pub surface: Surface<'a>,
    pub capabilities: SurfaceCapabilities,
}

impl<'a> Surfaces<'a> {
    pub fn insert(&mut self, uid: UID, window: Arc<Window>, gpu: &GPU) {
        let surface = gpu.instance.create_surface(window).unwrap();
        let capabilities = surface.get_capabilities(&gpu.adapter);
        self.surfaces.insert(
            uid,
            WindowSurface {
                surface,
                capabilities,
            },
        );
    }
    pub fn configure(&self, uid: &UID, device: &Device, size: (u32, u32)) {
        let Some(surface) = self.get(uid) else {
            return;
        };
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.capabilities.formats[0],
            view_formats: vec![surface.capabilities.formats[0].add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.0,
            height: size.1,
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
