use core::f32;

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
    pub fn generate_unit_points() -> Self {
        let points = vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0, 1.0],
            [0.0, -1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, -1.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        Verticies {
            velocities: vec![[0.0f32; 4]; points.len()],
            mass: vec![1.0; points.len()],
            points,
        }
    }
    pub fn generate_galaxy(
        max_radius: f32,
        max_phi: f32,
        star_count: usize,
        up: Vec3,
        gravitation_constant: f32
    ) -> Result<Self> {
        let mut rng = rng();
        let mut ret = Self {
            points: vec![[1.0; 4]; star_count],
            velocities: vec![[0.0; 4]; star_count],
            mass: vec![1.0; star_count]
        };
        // up DOT (x,y) = 0, up.x * x = -up.y * y
        // p = (x, (Up.x * x) / (-up.y))
        let up = up.normalize_or_zero();
        let r_axis = vec3(1.,1.,(up.x + up.y)/(-up.z)).normalize_or_zero();
        let phi_axis = up.cross(r_axis).normalize_or_zero();
        let galactic_mass = star_count as f32;

        for i in 0..star_count {
            let mut r: f32 = rng.random_range(0.0..=1.0);
            r = r.powf(1. / 3.) * max_radius; // 1/3 should be an even radial distribution, 1/2
                                              // biases towards center

            let theta = rng.random_range(0.0..f32::consts::PI * 2.);

            let mut phi = rng.random_range(-max_phi..=max_phi);
            phi *= phi.cos().powf(1./2. );

            let phi_rot = Quat::from_scaled_axis(phi_axis * phi);
            let theta_rot = Quat::from_scaled_axis(up * theta);

            let position = theta_rot.mul_vec3(phi_rot.mul_vec3(r_axis)) * r;

            ret.points[i][0] = position.x;
            ret.points[i][1] = position.y;
            ret.points[i][2] = position.z;

            let mut vel = theta_rot.mul_vec3(phi_axis);
            vel *= (gravitation_constant*galactic_mass / r).sqrt() * r/max_radius * 1.;

            ret.velocities[i][0] = vel.x;
            ret.velocities[i][1] = vel.y;
            ret.velocities[i][2] = vel.z;
        }

        ret.points[0] = [0.0,0.0,0.0,1.0];
        ret.velocities[0] = [0.0; 4];
        ret.mass[0] = star_count as f32 / 8.;

        Ok(ret)
    }
}
