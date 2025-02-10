#![allow(dead_code, unused_variables)]

use crate::graphics::vertices::Verticies;
use crate::prelude::*;

const INVERSE_SINGULARITY_THRESHOLD: f32 = 5e3;

pub fn physics_tick(delta: f32, vertices: &mut Verticies, gravitation_constant: f32) {
    let start = std::time::Instant::now();
    // Compute dv/dt, add to velocity
    for i1 in 0..vertices.points.len() {
        for i2 in 0..vertices.points.len() {
            if vertices.points[i1] != vertices.points[i2] {
                let p1 = vertices.points[i1];
                let p2 = vertices.points[i2];
                let d = vec3(p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]);
                let dlr = d.length_recip();
                if dlr < INVERSE_SINGULARITY_THRESHOLD * delta {
                    let f = gravitation_constant
                        * vertices.mass[i1]
                        * vertices.mass[i2]
                        * dlr.powi(3)
                        * d;
                    let a = f / vertices.mass[i1];
                    let dv = a * delta;
                    // info!("d: {:?}", d);
                    vertices.velocities[i1][0] += dv.x;
                    vertices.velocities[i1][1] += dv.y;
                    vertices.velocities[i1][2] += dv.z;
                }
            }
        }
    }

    // Add velocities
    for i in 0..vertices.points.len() {
        vertices.points[i][0] += vertices.velocities[i][0] * delta;
        vertices.points[i][1] += vertices.velocities[i][1] * delta;
        vertices.points[i][2] += vertices.velocities[i][2] * delta;
    }
    info!(
        "{:?}-Body Physics Tick: {:?} ms",
        vertices.points.len(),
        std::time::Instant::now()
            .duration_since(start)
            .as_secs_f64()
            / 1000.
    );
}
