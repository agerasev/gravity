use phy::{Rk4, Solver, System, Visitor};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use wgame::{glam::DVec2, rgb::Rgba};

use crate::body::{Body, Motion};

/// Fixed RK4 integration rate. Rendering may run at any frame rate.
pub const STEPS_PER_SECOND: u32 = 240;

pub struct Simulation {
    pub(crate) bodies: Vec<Body>,
    pub(crate) steps: u64,
    gravity: f64,
}

impl Simulation {
    /// Create the original 64-body distribution from a reproducible seed.
    pub fn new(seed: u64) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        Self {
            bodies: (0..64)
                .map(|_| {
                    Body::new(
                        DVec2::new(
                            rng.random_range(-400.0..400.0),
                            rng.random_range(-400.0..400.0),
                        ),
                        DVec2::new(
                            rng.random_range(-100.0..100.0),
                            rng.random_range(-100.0..100.0),
                        ),
                        10.0,
                        Rgba::new(rng.random(), rng.random(), rng.random(), 1.0),
                    )
                })
                .collect(),
            steps: 0,
            gravity: 1e5,
        }
    }

    pub fn time(&self) -> f64 {
        self.steps as f64 / f64::from(STEPS_PER_SECOND)
    }

    /// Advance one fixed step and record trail samples at their own cadence.
    pub fn step(&mut self) {
        Rk4.solve_step(self, 1.0 / STEPS_PER_SECOND as f32);
        self.steps += 1;
        for body in &mut self.bodies {
            body.trail.record(body.motion.position, self.steps);
        }
    }

    fn compute_gravity(&mut self) {
        for i in 0..self.bodies.len() {
            let (left, right) = self.bodies.split_at_mut(i);
            let body = &mut right[0];
            body.motion.deriv = Motion {
                position: body.motion.velocity,
                velocity: DVec2::ZERO,
            };
            for other in left {
                let delta = other.motion.position - body.motion.position;
                let distance = delta.length();
                // Coincident bodies have no preferred force direction. Avoid
                // the old 0/0 normalization while retaining the same force law.
                if distance > 0.0 {
                    let softening = 6.0 * (body.mass + other.mass);
                    let acceleration = (delta / distance) * self.gravity
                        / (distance * distance + softening * softening);
                    body.motion.deriv.velocity += acceleration;
                    other.motion.deriv.velocity -= acceleration;
                }
            }
        }
    }
}

impl System<Rk4> for Simulation {
    fn compute_derivs(&mut self, _: &<Rk4 as Solver>::Context) {
        self.compute_gravity();
    }

    fn visit_vars<V: Visitor<Rk4>>(&mut self, visitor: &mut V) {
        for body in &mut self.bodies {
            visitor.apply(&mut body.motion);
        }
    }
}

#[cfg(test)]
mod tests;
