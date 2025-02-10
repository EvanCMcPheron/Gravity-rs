#![allow(unused)]

use std::borrow::Borrow;

use wgpu::{
    core::device, hal::dx12::BindGroupLayout, util::RenderEncoder, FragmentState, TextureUsages,
};

use crate::prelude::*;

pub mod vertices;

#[derive(Debug)]
pub struct Graphics<'s> {
    surface: wgpu::Surface<'s>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertices: vertices::Verticies,
}

impl<'s> Graphics<'s> {
    fn generate_pipeline_layout(device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        })
    }
    fn generate_render_pipeline(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
    ) -> wgpu::RenderPipeline {
        let shaders = device.create_shader_module(include_wgsl!("../../shaders/render.wgsl"));
        let surface_format = surface.get_capabilities(&adapter).formats[0];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&Self::generate_pipeline_layout(device)),
            vertex: wgpu::VertexState {
                module: &shaders,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[vertices::Verticies::get_vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(surface_format.into())],
            }),
            // Specify point primitives
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
                ..Default::default()
            },
            cache: None,
            multiview: None,
            depth_stencil: None,
            multisample: Default::default(),
        })
    }
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

        let vertices = vertices::Verticies {
            points: vec![
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                // [-0.5, -0.5, 0.0, 0.0],
                // [0.5, -0.5, 0.0, 0.0],
                // [-0.5, 0.5, 0.0, 0.0],
                // [0.5, 0.5, 0.0, 0.0],
            ],
            velocities: vec![],
            mass: vec![],
        };

        Ok(Graphics {
            render_pipeline: Self::generate_render_pipeline(&device, &surface, &adapter),
            surface,
            adapter,
            device,
            queue,
            surface_config,
            vertices,
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
                resolve_target: None;,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(crate::CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        };

        let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);

        rpass.set_pipeline(&self.render_pipeline);

        rpass.set_vertex_buffer(
            0,
            self.vertices.create_vertex_buffer(&self.device).slice(..),
        );

        rpass.draw(0..4, 0..1);

        drop(rpass);

        self.queue.submit(Some(command_encoder.finish()));

        surface_tex.present();

        Ok(())
    }
}
