#![allow(dead_code, unused_variables)]
use std::arch::x86_64;

use crate::graphics::{
    rendering::{Camera, ViewModeLookAt, ViewModeLookTo},
    Graphics,
};
use crate::prelude::*;

use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop,
    window::{self, Window},
};

#[derive(Debug)]
pub struct UserOptions {
    pub mouse_sensitivity: f32,
    pub line_size: f32,
    pub scroll_sensitivity: f32,
}

impl Default for UserOptions {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.7,
            line_size: 5.,
            scroll_sensitivity: 7.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct CursorState {
    delta: (f64, f64),
    scroll_delta: Vec2,
    presed: bool,
}

impl CursorState {
    pub fn add_delta(&mut self, delta: (f64, f64)) {
        self.delta.0 += delta.0;
        self.delta.1 += delta.1;
    }
    pub fn pop_delta(&mut self) -> (f64, f64) {
        let ret = self.delta;
        self.delta = (0., 0.);
        ret
    }
    pub fn add_scroll_delta(&mut self, delta: Vec2) {
        self.scroll_delta += delta
    }
    pub fn pop_scroll_delta(&mut self) -> Vec2 {
        let ret = self.scroll_delta;
        self.scroll_delta = Vec2::ZERO;
        ret
    }
    pub fn set_pressed(&mut self, pressed: bool) {
        self.presed = pressed;
    }
}

#[derive(Debug, Default)]
pub struct App<'app> {
    window: Option<Arc<Window>>,
    graphics: Option<crate::graphics::Graphics<'app>>,
    camera: Option<Camera<ViewModeLookAt>>,
    previous_frame: Option<std::time::Instant>,
    cursor_state: CursorState,
    options: UserOptions,
}

impl<'app> App<'app> {
    fn create_camera(aspect_ratio: f32) -> Camera<ViewModeLookAt> {
        Camera::<ViewModeLookAt>::new(
            vec3(0., 0., 0.),
            vec3(0.5, 0.5, 0.5) * 5.,
            Vec3::Y,
            2.,
            aspect_ratio,
            0.05,
        )
    }
    fn process_frame(&mut self, delta: f32) -> Result<()> {
        info!("FPS: {:?}", 1./delta);
        if self.cursor_state.presed {
            let cursor_delta = self.cursor_state.pop_delta();
            let cursor_delta = vec2(cursor_delta.0 as f32, cursor_delta.1 as f32);

            let mut up = self.camera.as_ref().unwrap().up;
            if !up.is_normalized() {
                up = up.normalize_or_zero();
                self.camera.as_mut().unwrap().up = up;
            }

            let mut pitch_dir = up.cross(self.camera.as_ref().unwrap().get_orientation());
            if !pitch_dir.is_normalized() {
                pitch_dir = pitch_dir.normalize_or_zero();
            }

            let mut rotation_vec = -up * cursor_delta.x * self.options.mouse_sensitivity * delta;
            rotation_vec += pitch_dir * cursor_delta.y * self.options.mouse_sensitivity * delta;

            self.camera
                .as_mut()
                .unwrap()
                .rotate(Quat::from_scaled_axis(rotation_vec));
        }
        let scroll_delta = self.cursor_state.pop_scroll_delta();
        if !(scroll_delta.y == 0. || self.options.scroll_sensitivity == 0.) {
            let scroll = scroll_delta.y * delta * self.options.scroll_sensitivity + 1.; // if (scroll_delta.y > 0.) {scroll_delta.y} else {1./scroll_delta.y};
            self.camera.as_mut().unwrap().zoom(scroll);
        }

        Ok(())
    }
}

impl<'app> ApplicationHandler for App<'app> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attr = Window::default_attributes().with_title("Gravity Simulation");

        self.window = Some(Arc::new(
            event_loop
                .create_window(attr)
                .with_context(|| "failed to create a window")
                .unwrap(),
        ));
        let size = self.window.as_ref().unwrap().inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        self.camera = Some(Self::create_camera(aspect_ratio));
        self.previous_frame = Some(std::time::Instant::now());
        self.graphics = Some(
            Graphics::new(self.window.as_ref().unwrap().clone())
                .with_context(|| "failed to create window")
                .unwrap(),
        );
    }
    fn device_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        device_id: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        match event {
            event::DeviceEvent::MouseMotion { delta } => {
                self.cursor_state.add_delta(delta);
            }
            event::DeviceEvent::MouseWheel { delta } => {
                self.cursor_state.add_scroll_delta(match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        vec2(x * self.options.line_size, y * self.options.line_size)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(d) => vec2(d.x as f32, d.y as f32),
                });
            }
            _ => {}
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if let winit::event::MouseButton::Left = button {
                    self.cursor_state.set_pressed(state.is_pressed());
                    // Clear current delta, so there isn't a jump every time
                    // there is a left click
                    let _ = self.cursor_state.pop_delta();
                }
            }
            WindowEvent::Resized(size) => {
                self.graphics.as_mut().unwrap().resize(size);
                // self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let delta = now
                    .duration_since(self.previous_frame.unwrap())
                    .as_secs_f32();

                self.process_frame(delta).unwrap();
                self.graphics
                    .as_mut()
                    .unwrap()
                    .render(self.camera.as_ref().unwrap())
                    .unwrap();
                self.window.as_ref().unwrap().request_redraw();
                self.previous_frame = Some(now);
            }
            WindowEvent::CloseRequested => {
                info!("Close Requested, closing...");
                event_loop.exit();
            }
            _ => (),
        }
    }
}
