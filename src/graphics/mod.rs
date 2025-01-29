#![allow(unused)]

use std::borrow::Borrow;

use wgpu::{core::device, TextureUsages};

use crate::prelude::*;

#[derive(Debug)]
pub struct Graphics<'s> {
    surface: wgpu::Surface<'s>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration
}

impl<'s> Graphics<'s> {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&Default::default());

        let mut surface = instance
            .create_surface(window.clone())
            .with_context(|| "Failed to create surface from window")?;

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: true,
        }))
        .ok_or_else(|| anyhow!("Failed to create"))?;

        let (device, queue) = block_on(adapter.request_device(&Default::default(), None))
            .with_context(|| "Failed to obtain device")?;

        let size = window.clone().inner_size();

        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or_else(|| anyhow!("faild to create surface configuration"))?;

        surface.configure(&device, &surface_config);

        Ok(Graphics {
            surface,
            adapter,
            device,
            queue,
            surface_config
        })
    }
    fn reconfigure_surface(&self) {
        self.surface.configure(&self.device, &self.surface_config);
    }
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.reconfigure_surface();
    }
    pub fn render(&mut self) -> Result<()> {
        let surface_tex = self
            .surface
            .get_current_texture()
            .with_context(|| "Failed to get current surface texture")?;

        let view = surface_tex.texture.create_view(&Default::default());

        let mut command_encoder = self.device.create_command_encoder(&Default::default());

        let render_pass_descriptor = &wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(crate::CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        };

        let rpass = command_encoder.begin_render_pass(&render_pass_descriptor);

        drop(rpass);

        self.queue.submit(Some(command_encoder.finish()));

        surface_tex.present();

        Ok(())
    }
}
