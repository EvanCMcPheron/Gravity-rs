use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Verticies {
    /// The actual positions of the points
    pub points: Vec<[f32; 4]>,
    /// The velocities of the points
    pub velocities: Vec<[f32; 4]>,
    pub mass: Vec<f32>,
}

impl Verticies {
    pub fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vec4>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4],
        }
    }
    pub fn create_vertex_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Points Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.points),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}
