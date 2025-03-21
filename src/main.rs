#![allow(dead_code, unused_variables)]
const LOG_LEVEL: simplelog::LevelFilter = simplelog::LevelFilter::Trace;
const CLEAR_COLOR: wgpu::Color = wgpu::Color::BLACK;

mod application;
mod graphics;

pub mod prelude {
    pub use anyhow::{anyhow, bail, Context, Result};
    pub use async_std::task::block_on;
    pub use bytemuck::{Pod, Zeroable};
    pub use derive_builder::Builder;
    pub use glam::{Mat4, Vec3, Vec4, Quat, vec3, vec4, vec2, Vec2};
    pub use log::{debug, error, info, trace, warn};
    pub use std::default::Default;
    pub use std::sync::Arc;
    pub use wgpu::include_wgsl;
    pub use wgpu::util::DeviceExt;
    pub use rand::{
        Rng,
        rng
    };
    pub use std::sync::atomic::*;
}


use prelude::*;

fn main() {
    let file = std::fs::File::create("LOG")
        .with_context(|| "Failed to create LOG file")
        .unwrap();
    simplelog::WriteLogger::init(LOG_LEVEL, Default::default(), file)
        .with_context(|| "Failed to create logger")
        .unwrap();

    let event_loop = winit::event_loop::EventLoop::new()
        .with_context(|| "Failed to create event loop")
        .unwrap();

    let mut app: application::App = Default::default();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop
        .run_app(&mut app)
        .with_context(|| "Running app failed")
        .unwrap();
}
