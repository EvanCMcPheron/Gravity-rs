#![allow(unused)]

use crate::prelude::*;

use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop,
    window::{self, Window},
};

#[derive(Debug, Default)]
pub struct App<'app> {
    window: Option<Arc<Window>>,
    graphics: Option<crate::graphics::Graphics<'app>>,
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
        self.graphics = Some(
            crate::graphics::Graphics::new(self.window.as_ref().unwrap().clone())
                .with_context(|| "failed to create window")
                .unwrap(),
        );
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.graphics.as_mut().unwrap().resize(size);
                // self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.graphics.as_mut().unwrap().render();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CloseRequested => {
                info!("Close Requested, closing...");
                event_loop.exit();
            }
            _ => (),
        }
    }
}
