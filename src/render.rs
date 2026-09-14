use wgame::{Library, gfx::Scene, prelude::*};

use crate::Simulation;

impl Simulation {
    pub fn draw(&self, library: &Library, scene: &mut Scene) {
        let mut points = Vec::with_capacity(34);
        let shapes = library.shapes();
        for body in &self.bodies {
            body.trail
                .write_points(body.motion.position, self.steps, body.radius, &mut points);
            let mut color = body.color;
            color.a *= 0.5;
            scene.add(&shapes.polyline(&points).fill_color(color));
        }
        for body in &self.bodies {
            scene.add(
                &shapes
                    .unit_circle()
                    .scale(body.radius as f32)
                    .move_to(body.motion.position.as_vec2())
                    .fill_color(body.color),
            );
        }
    }
}
