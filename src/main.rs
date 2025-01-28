const LOG_LEVEL: simplelog::LevelFilter = simplelog::LevelFilter::Info;

mod application;

pub mod prelude {
    pub use anyhow::{anyhow, bail, Context, Result};
    pub use log::{debug, error, info, trace, warn};
    pub use std::default::Default;
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
