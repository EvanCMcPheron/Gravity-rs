#![allow(dead_code, unused_variables)]
use std::{borrow::Borrow, time::Duration};

use rendering::ViewMode;

use crate::prelude::*;

pub mod compute;
pub mod rendering;
pub mod vertices;

use vertices::{BodyData, Compute};

#[derive(Debug)]
pub struct Graphics<'s> {
    surface: wgpu::Surface<'s>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    body_data: BodyData<Compute>,
}

impl<'s> Graphics<'s> {
    fn generate_depth_stencil_state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            bias: wgpu::DepthBiasState::default(),
            depth_compare: wgpu::CompareFunction::Less,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
        }
    }
    fn generate_depth_texture(&self) -> wgpu::TextureView {
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Stencil"),
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = self.device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
    fn generate_pipeline_layout(device: &wgpu::Device) -> wgpu::PipelineLayout {
        let bg_0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[rendering::Uniform::generate_bind_group_layout_entry(
                device, 0,
            )],
        });
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bg_0],
            push_constant_ranges: &[],
        })
    }
    fn generate_compute_bind_groups(&self) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.body_data.positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.body_data.velocities.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.body_data.mass.as_entire_binding(),
                }
            ]
        })
    }
    fn generate_compute_pipeline_layout(device: &wgpu::Device) -> wgpu::PipelineLayout {
        // @Todo | update min_binding_size to account for length of buffers,
        // pontentially optomising bind group allocation
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        },
                        count: None //Some(std::num::NonZero::new(self.body_data.len as u32).unwrap())
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: false
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: false
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            })]
        })
    }
    fn generate_compute_pipeline(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> wgpu::ComputePipeline {
        let module = device.create_shader_module(include_wgsl!("../../shaders/compute.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&Self::generate_compute_pipeline_layout(device)),
            module: &module,
            entry_point: Some("cs_entry"),
            compilation_options: Default::default(),
            cache: None
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
                buffers: &[BodyData::<Compute>::get_vertex_buffer_layout()],
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
            depth_stencil: Some(Self::generate_depth_stencil_state()),
            multisample: Default::default(),
        })
    }
    pub fn new(window: Arc<winit::window::Window>, instance: wgpu::Instance) -> Result<Self> {
        use std::{collections::HashMap, time::Instant};

        let mut start = Instant::now();
        let mut times: HashMap<&'static str, Duration> = HashMap::new();

        // let instance = wgpu::Instance::new(&Default::default());
        // times.insert("Instance Creation", start.elapsed());
        // start = Instant::now();

        let surface = instance
            .create_surface(window.clone())
            .with_context(|| "Failed to create surface from window")?;
        times.insert("Surface Creation", start.elapsed());
        start = Instant::now();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow!("Failed to create"))?;
        times.insert("Adapter Creation", start.elapsed());
        start = Instant::now();

        let (device, queue) = block_on(adapter.request_device(&Default::default(), None))
            .with_context(|| "Failed to obtain device")?;
        times.insert("Device Creation", start.elapsed());
        start = Instant::now();

        let size = window.clone().inner_size();

        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or_else(|| anyhow!("faild to create surface configuration"))?;

        surface.configure(&device, &surface_config);
        times.insert("Surface configuration", start.elapsed());
        start = Instant::now();

        let mut encoder = device.create_command_encoder(&Default::default());
        times.insert("Encoder Creation", start.elapsed());
        start = Instant::now();
        let body_data = BodyData::<Compute>::generate_unit_points(&device, &mut encoder)
            .with_context(|| "Failed to create unit points")?;
        times.insert("Creating Body Data", start.elapsed());
        start = Instant::now();
        queue.submit(Some(encoder.finish()));
        times.insert("Queue submission", start.elapsed());
        // start = Instant::now();

        info!("Graphics Instanciation Times - {:?}", times);

        Ok(Graphics {
            render_pipeline: Self::generate_render_pipeline(&device, &surface, &adapter),
            compute_pipeline: Self::generate_compute_pipeline(&device, &adapter),
            surface,
            adapter,
            device,
            queue,
            surface_config,
            body_data,
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
    fn create_uniform_bind_group(&self, uniform: rendering::Uniform) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[rendering::Uniform::generate_bind_group_layout_entry(
                        &self.device,
                        0,
                    )],
                }),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.generate_buffer(&self.device).as_entire_binding(),
            }],
        })
    }
    pub fn render<M: ViewMode + Default>(&mut self, camera: &rendering::Camera<M>) -> Result<()> {
        let uniform = rendering::UniformBuilder::default()
            .height(self.surface_config.height)
            .width(self.surface_config.width)
            .world_mat(camera.generate_world_matrix_columns())
            .build()
            .with_context(|| "Failed to generate Uniform Struct from UniformBuilder")?;

        let surface_tex = self
            .surface
            .get_current_texture()
            .with_context(|| "Failed to get current surface texture")?;

        let view = surface_tex.texture.create_view(&Default::default());

        let mut command_encoder = self.device.create_command_encoder(&Default::default());

        let depth_texture = self.generate_depth_texture();

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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        };

        {
            let mut cpass = command_encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0 ,&self.generate_compute_bind_groups(), &[]);
            cpass.dispatch_workgroups(self.body_data.len as u32, self.body_data.len as u32, 1);
        }

        {
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);

            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_bind_group(0, &self.create_uniform_bind_group(uniform), &[]);

            rpass.set_vertex_buffer(0, self.body_data.positions.slice(..));

            rpass.draw(0..(self.body_data.len as u32), 0..1);
        }

        self.queue.submit(Some(command_encoder.finish()));


        surface_tex.present();

        Ok(())
    }
}
