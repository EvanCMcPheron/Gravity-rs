#![allow(unused)]

use crate::prelude::*;

use winit::{
    application::ApplicationHandler, event::{
        self,
        WindowEvent,
    }, event_loop, window::{self, Window}
};

#[derive(Debug, Default)]
pub struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attr = Window::default_attributes().with_title("Gravity Simulation");
        self.window = Some(
            event_loop
                .create_window(attr)
                .with_context(|| "failed to create a window")
                .unwrap(),
        )
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                todo!();
                self.window.unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                todo!();
                self.window.unwrap().request_redraw();
            }
            WindowEvent::CloseRequested => {
                info!("Close Requested, closing...");
                event_loop.exit();
            },
            _ => ()
        }
    }
}
