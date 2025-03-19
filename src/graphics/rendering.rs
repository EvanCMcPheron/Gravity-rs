use bytemuck::bytes_of;

use crate::prelude::*;

#[derive(Builder, Debug, Default)]
pub struct Camera<M: ViewMode + Default> {
    pub position: Vec3,
    view_mode: M,
    pub up: Vec3,
    /// The fov in the x direction, seems more intuitive but requires conversion to fov_y
    pub fov: f32,
    /// width/height
    pub aspect_ratio: f32,
    pub z_near: f32,
}

pub trait ViewMode {
    fn generate_view_matrix(&self, position: Vec3, up: Vec3) -> Mat4;

    /// For defining different rotation behaviour between LookAt and LookTo
    /// LookTo should rotate the direction, but look at should do nothing internally
    /// as will be defined by the position of the parent camera relative to the
    /// focal point, so the parent's psoition (&mut position), should be mutated
    fn rotate_orientation(&mut self, rotation: Quat, position: &mut Vec3);

    /// Creates a view mode given a camera position and orientation, since
    /// implimentation varies between the modes
    fn with_position_orientation(position: Vec3, orientation: Vec3) -> (Self, Vec3)
    where
        Self: Sized;
    fn get_position(&self, camera_position: Vec3) -> Vec3;
    fn set_position(&mut self, new_position: Vec3, camera_position: &mut Vec3);
    fn get_orientation(&self, camera_position: Vec3) -> Vec3;
    fn set_orientation(&mut self, new_orientation: Vec3, camera_position: &mut Vec3);
    fn zoom(&mut self, zoom: f32, camera_position: &mut Vec3, fov: &mut f32);
}

#[derive(Debug, Default)]
pub struct ViewModeLookAt {
    focus: Vec3,
}

#[derive(Debug, Default)]
pub struct ViewModeLookTo {
    direction: Vec3,
}

impl ViewMode for ViewModeLookTo {
    fn generate_view_matrix(&self, position: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_to_rh(position, self.direction, up)
    }
    /// Simply multiplys the quatinary rotation instruction with he internal
    /// direction field
    fn rotate_orientation(&mut self, rotation: Quat, position: &mut Vec3) {
        self.direction = rotation * self.direction;
    }
    fn with_position_orientation(position: Vec3, orientation: Vec3) -> (Self, Vec3)
    where
        Self: Sized,
    {
        (
            Self {
                direction: orientation,
            },
            position,
        )
    }
    fn get_orientation(&self, camera_position: Vec3) -> Vec3 {
        self.direction
    }
    fn set_orientation(&mut self, new_orientation: Vec3, camera_position: &mut Vec3) {
        self.direction = new_orientation
    }
    fn get_position(&self, camera_position: Vec3) -> Vec3 {
        camera_position
    }
    fn set_position(&mut self, new_position: Vec3, camera_position: &mut Vec3) {
        *camera_position = new_position
    }
    fn zoom(&mut self, zoom: f32, camera_position: &mut Vec3, fov: &mut f32) {
        *fov /= zoom
    }
}

impl ViewMode for ViewModeLookAt {
    fn generate_view_matrix(&self, position: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(position, self.focus, up)
    }
    /// rotations the relative position of the parent camera around the focal
    /// point. The focal point is not mutated
    /// Does 0 internal mutation, so the &mut self is just for conforming
    /// to the ViewMode trait signature.
    fn rotate_orientation(&mut self, rotation: Quat, position: &mut Vec3) {
        // d = p - f
        let mut dif = position.clone() - self.focus;
        // mutate d
        dif = rotation * dif;
        // p = d + f
        *position = self.focus + dif;
    }
    fn with_position_orientation(position: Vec3, orientation: Vec3) -> (Self, Vec3)
    where
        Self: Sized,
    {
        (Self { focus: position }, position - orientation)
    }
    fn get_orientation(&self, camera_position: Vec3) -> Vec3 {
        self.focus - camera_position
    }
    fn set_orientation(&mut self, new_orientation: Vec3, camera_position: &mut Vec3) {
        *camera_position = self.focus - new_orientation;
    }
    fn get_position(&self, camera_position: Vec3) -> Vec3 {
        self.focus
    }
    fn set_position(&mut self, new_position: Vec3, camera_position: &mut Vec3) {
        self.focus = new_position;
    }
    fn zoom(&mut self, zoom: f32, camera_position: &mut Vec3, fov: &mut f32) {
        info!("received zoom: {:?}", zoom);
        let mut dif = camera_position.clone() - self.focus;
        dif = zoom * dif;
        *camera_position = self.focus + dif;
        
    }
}

impl<M: ViewMode + Default> Camera<M> {
    fn generate_view_matrix(&self) -> Mat4 {
        self.view_mode.generate_view_matrix(self.position, self.up)
    }
    fn generate_perspective_matrix(&self) -> Mat4 {
        // the fov multiplcation occurs because self.fov is for x dimension,
        // so fov/aspect_ratio
        Mat4::perspective_infinite_rh(self.fov / self.aspect_ratio, self.aspect_ratio, self.z_near)
    }
    fn generate_world_matix(&self) -> Mat4 {
        self.generate_perspective_matrix().mul_mat4( &self.generate_view_matrix())
    }
    pub fn generate_world_matrix_columns(&self) -> [[f32; 4]; 4] {
        self.generate_world_matix().to_cols_array_2d()
    }
    /// This is created as it has slightly different behaviour based on the view
    /// mode type
    ///
    /// ViewModeLookAt: The position is set to the focus of the view mode, as
    /// when the camera is moved in this mode it should move the focal point,
    /// not the camera position. The actual position of the camera is the
    /// psoition - orientation, so in this case the *magnitude* of orientation
    /// **matters**
    ///
    /// ViewmodeLookTo: The position and orientation of the camera are what they
    /// sound like they are, length of orientation does not matter.
    pub fn new(
        position: Vec3,
        orientation: Vec3,
        up: Vec3,
        fov: f32,
        aspect_ratio: f32,
        z_near: f32,
    ) -> Self {
        // Self {
        //     position,
        //     view_mode: view,
        //     up,
        //     fov,
        //     aspect_ratio,
        //     z_near,
        // }

        let (view_mode, position) = M::with_position_orientation(position, orientation);
        Self {
            position,
            view_mode,
            up,
            fov,
            aspect_ratio,
            z_near,
        }
    }
    pub fn get_orientation(&self) -> Vec3 {
        self.view_mode.get_orientation(self.position)
    }
    pub fn set_orientation(&mut self, new_orientation: Vec3) {
        self.view_mode
            .set_orientation(new_orientation, &mut self.position);
    }
    pub fn get_position(&self) -> Vec3 {
        self.view_mode.get_position(self.position)
    }
    pub fn set_position(&mut self, new_position: Vec3) {
        self.view_mode
            .set_position(new_position, &mut self.position);
    }
    pub fn rotate(&mut self, rotation: Quat) {
        self.up = rotation * self.up;
        self.view_mode
            .rotate_orientation(rotation, &mut self.position);
    }
    pub fn zoom(&mut self, zoom: f32) {
        self.view_mode.zoom(zoom, &mut self.position, &mut self.fov);
    }
}

#[derive(Builder, Default, Debug, Pod, Zeroable, Copy, Clone)]
#[repr(C)]
pub struct Uniform {
    pub world_mat: [[f32; 4]; 4],
    pub width: u32,
    pub height: u32,
    #[builder(setter(skip))]
    padding: [f32; 2],
}

impl Uniform {
    pub fn generate_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytes_of(self),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
    pub fn generate_bind_group_layout_entry(
        device: &wgpu::Device,
        binding: u32,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}
