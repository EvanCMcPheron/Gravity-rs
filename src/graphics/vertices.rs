use core::f32;

use wgpu::Maintain;

use crate::prelude::*;

#[derive(Debug)]
pub struct BodyData<B: BufferType> {
    /// The actual positions of the points
    pub positions: Arc<wgpu::Buffer>,
    /// The velocities of the points
    pub velocities: Arc<wgpu::Buffer>,
    pub mass: Arc<wgpu::Buffer>,
    pub len: usize,
    buffer_type: B,
}

#[derive(Debug, Default)]
pub struct UnbufferedBodyData {
    pub positions: Arc<Vec<[f32; 4]>>,
    pub velocities: Arc<Vec<[f32; 4]>>,
    pub mass: Arc<Vec<f32>>
}

impl BodyData<Compute> {
    pub fn copy_from_mappable(&self, mappable: &BodyData<Mappable>, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        mappable.ensure_mapping_complete(device);
        let size = mappable.positions.size();
        encoder.copy_buffer_to_buffer(mappable.positions.as_ref(), 0_u64, self.positions.as_ref(), 0_u64, size);
        let size = mappable.velocities.size();
        encoder.copy_buffer_to_buffer(mappable.positions.as_ref(), 0_u64, self.velocities.as_ref(), 0_u64, size);
        let size = mappable.mass.size();
        encoder.copy_buffer_to_buffer(mappable.mass.as_ref(), 0_u64, self.mass.as_ref(), 0_u64, size);
    }
    pub fn map_to(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, data: &UnbufferedBodyData) -> Result<()> {
        let mut mapping_buffer = BodyData::<Mappable>::with_length(device, self.len);
        
        mapping_buffer.map(data)?;
        self.copy_from_mappable(&mapping_buffer, device, encoder);
        Ok(())
    }
}

impl BodyData<Mappable> {
    pub fn map(&mut self, data: &UnbufferedBodyData) -> Result<()> {
        if !(data.positions.len() == data.velocities.len() && data.velocities.len() == data.mass.len()) {
            bail!("The lengths of the data fields do not equal eachother")
        }
        if data.mass.len() != self.len {
            bail!("The length of the data fields does not equal the buffer length")
        }

        let (c_position, c_velocities, c_mass) = (
            self.positions.clone(),
            self.velocities.clone(),
            self.mass.clone(),
        );

        self.buffer_type.mapped_buffer_count.store(0, Ordering::Relaxed);

        let positions = data.positions.clone();
        let p_atomic = self.buffer_type.mapped_buffer_count.clone();
        self.positions.slice(..).map_async(wgpu::MapMode::Write, move |map_result| {
            if map_result.is_ok() {
                c_position.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(&positions[..]));
                let prev_value = p_atomic.load(Ordering::Relaxed);
                p_atomic.store(prev_value + 1, Ordering::Relaxed);
            } else {
                panic!("Failed to map positions buffer")
            }
        });

        let velocities = data.velocities.clone();
        let v_atomic = self.buffer_type.mapped_buffer_count.clone();
        self.velocities.slice(..).map_async(wgpu::MapMode::Write, move |map_result| {
            if map_result.is_ok() {
                c_velocities.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(&velocities[..]));
                let prev_value = v_atomic.load(Ordering::Relaxed);
                v_atomic.store(prev_value + 1, Ordering::Relaxed);
            } else {
                panic!("Failed to map velocities buffer")
            }
        });

        let mass = data.mass.clone();
        let m_atomic = self.buffer_type.mapped_buffer_count.clone();
        self.mass.slice(..).map_async(wgpu::MapMode::Write, move |map_result| {
            if map_result.is_ok() {
                c_mass.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(&mass[..]));
                let prev_value = m_atomic.load(Ordering::Relaxed);
                m_atomic.store(prev_value + 1, Ordering::Relaxed);
            } else {
                panic!("Failed to map mass buffer")
            }
        });

        Ok(())
    }
    fn ensure_mapping_complete(&self, device: &wgpu::Device) {
        if self.buffer_type.mapped_buffer_count.load(Ordering::Relaxed) < 3 {
            device.poll(Maintain::Wait);
        }
    }
}


impl<B: BufferType> BodyData<B> {
    fn create_buffer_desc(
        unit_size: usize,
        len: usize,
        usage: wgpu::BufferUsages,
    ) -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: std::mem::size_of::<f32>() as u64 * len as u64 * unit_size as u64,
            usage,
        }
    }
    pub fn with_length(device: &wgpu::Device, len: usize) -> BodyData<B> {
        let vector_buffer_desc = Self::create_buffer_desc(4, len, B::get_usages());

        let mass_buffer_desc = Self::create_buffer_desc(1, len, B::get_usages());

        BodyData::<B> {
            positions: Arc::new(device.create_buffer(&vector_buffer_desc)),
            velocities: Arc::new(device.create_buffer(&vector_buffer_desc)),
            mass: Arc::new(device.create_buffer(&mass_buffer_desc)),
            len,
            buffer_type: B::new(),
        }
    }
    pub fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<[f32; 4]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4],
        }
    }
    pub fn generate_unit_points(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> Result<BodyData<Compute>> {
        let points = Arc::new(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0, 1.0],
            [0.0, -1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, -1.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let velocities = Arc::new(vec![[0.0; 4]; points.len()]);
        let masses = Arc::new(vec![1.0; points.len()]);

        let compute = BodyData::<Compute>::with_length(device, points.len());
        compute.map_to(device, encoder, &UnbufferedBodyData {
            positions: points,
            velocities,
            mass: masses
        }).with_context(|| "Failed to map unit points buffers to gpu bound buffers")?;
        Ok(compute)
    }
    // pub fn generate_galaxy(
    //     max_radius: f32,
    //     max_phi: f32,
    //     star_count: usize,
    //     up: Vec3,
    //     gravitation_constant: f32,
    // ) -> Result<Self> {
    //     let mut rng = rng();
    //     let mut ret = Self {
    //         points: vec![[1.0; 4]; star_count],
    //         velocities: vec![[0.0; 4]; star_count],
    //         mass: vec![1.0; star_count],
    //     };
    //     // up DOT (x,y) = 0, up.x * x = -up.y * y
    //     // p = (x, (Up.x * x) / (-up.y))
    //     let up = up.normalize_or_zero();
    //     let r_axis = vec3(1., 1., (up.x + up.y) / (-up.z)).normalize_or_zero();
    //     let phi_axis = up.cross(r_axis).normalize_or_zero();
    //     let galactic_mass = star_count as f32;
    //
    //     for i in 0..star_count {
    //         let mut r: f32 = rng.random_range(0.0..=1.0);
    //         r = r.powf(1. / 3.) * max_radius; // 1/3 should be an even radial distribution, 1/2
    //                                           // biases towards center
    //
    //         let theta = rng.random_range(0.0..f32::consts::PI * 2.);
    //
    //         let mut phi = rng.random_range(-max_phi..=max_phi);
    //         phi *= phi.cos().powf(1. / 2.);
    //
    //         let phi_rot = Quat::from_scaled_axis(phi_axis * phi);
    //         let theta_rot = Quat::from_scaled_axis(up * theta);
    //
    //         let position = theta_rot.mul_vec3(phi_rot.mul_vec3(r_axis)) * r;
    //
    //         ret.points[i][0] = position.x;
    //         ret.points[i][1] = position.y;
    //         ret.points[i][2] = position.z;
    //
    //         let mut vel = theta_rot.mul_vec3(phi_axis);
    //         vel *= (gravitation_constant * galactic_mass / r).sqrt() * r / max_radius * 1.;
    //
    //         ret.velocities[i][0] = vel.x;
    //         ret.velocities[i][1] = vel.y;
    //         ret.velocities[i][2] = vel.z;
    //     }
    //
    //     ret.points[0] = [0.0, 0.0, 0.0, 1.0];
    //     ret.velocities[0] = [0.0; 4];
    //     ret.mass[0] = star_count as f32 / 8.;
    //
    //     Ok(ret)
    // }
}

pub trait BufferType {
    fn get_usages() -> wgpu::BufferUsages;
    fn new() -> Self;
}

#[derive(Debug, Default)]
pub struct Mappable {
    pub mapped_buffer_count: Arc<AtomicUsize>
}

#[derive(Debug, Default)]
pub struct Compute;

impl BufferType for Mappable {
    fn get_usages() -> wgpu::BufferUsages {
        use wgpu::BufferUsages as BU;
        BU::MAP_WRITE | BU::COPY_SRC
    }
    fn new() -> Self {
        Self {
            mapped_buffer_count: Arc::new(AtomicUsize::new(3))
        }
    }
}

impl BufferType for Compute {
    fn get_usages() -> wgpu::BufferUsages {
        use wgpu::BufferUsages as BU;
        BU::INDIRECT | BU::VERTEX | BU::STORAGE | BU::COPY_DST
    }
    fn new() -> Self {
        Self
    }
}
