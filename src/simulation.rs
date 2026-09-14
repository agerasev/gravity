use phy::{Rk4, Solver, System, Visitor};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use wgame::{glam::DVec2, rgb::Rgba};

use crate::body::{Body, Motion};

/// Fixed RK4 integration rate. Rendering may run at any frame rate.
pub const STEPS_PER_SECOND: u32 = 240;
const GRAVITY: f64 = 120.0;
const SOFTENING: f64 = 8.0;

/// A body's initial conditions, in deliberately fictional world units.
#[derive(Clone, Copy, Debug)]
pub struct BodySpec {
    pub mass: f64,
    pub color: Rgba<f32>,
    pub velocity: DVec2,
}

impl BodySpec {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.mass.is_finite() || !(0.01..=100_000.0).contains(&self.mass) {
            return Err("Mass must be 0.01 to 100000");
        }
        if !self.velocity.is_finite() || self.velocity.abs().max_element() > 10_000.0 {
            return Err("Velocity must be -10000 to 10000");
        }
        if [self.color.r, self.color.g, self.color.b, self.color.a]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("Color channels must be 0 to 1");
        }
        Ok(())
    }
}

/// Exaggerated visual radius; bodies are point masses and do not collide.
pub fn body_radius(mass: f64) -> f64 {
    2.5 + 1.7 * mass.cbrt()
}

pub struct Simulation {
    pub(crate) bodies: Vec<Body>,
    pub(crate) steps: u64,
    gravity: f64,
}

impl Simulation {
    /// Compact, approximate circular orbits, with zero net momentum.
    pub fn solar_system() -> Self {
        let star_mass = 4000.0;
        let mut sim = Self {
            bodies: vec![Body::new(
                DVec2::ZERO,
                DVec2::ZERO,
                star_mass,
                Rgba::new(1.0, 0.78, 0.25, 1.0),
            )],
            steps: 0,
            gravity: GRAVITY,
        };
        for (radius, mass, angle, rgb) in [
            (75.0_f64, 0.3, 0.4_f64, [0.7, 0.65, 0.6]),
            (115.0, 0.7, 2.2, [1.0, 0.72, 0.4]),
            (165.0, 1.0, 4.1, [0.3, 0.65, 1.0]),
            (215.0, 0.5, 0.8, [1.0, 0.36, 0.24]),
            (340.0, 20.0, 3.3, [0.85, 0.64, 0.45]),
            (460.0, 10.0, 5.4, [0.95, 0.84, 0.56]),
            (600.0, 2.0, 1.9, [0.38, 0.86, 0.9]),
        ] {
            let radial = DVec2::new(angle.cos(), angle.sin());
            let speed = (GRAVITY * star_mass * radius.powi(2)
                / (radius.powi(2) + SOFTENING.powi(2)).powf(1.5))
            .sqrt();
            sim.bodies.push(Body::new(
                radial * radius,
                DVec2::new(radial.y, -radial.x) * speed,
                mass,
                Rgba::new(rgb[0], rgb[1], rgb[2], 1.0),
            ));
        }
        // Keep moon orbits well inside their host's sphere of influence.
        for (host, radius, angle, rgb) in [
            (5, 15.0_f64, 1.2_f64, [0.9, 0.87, 0.78]),
            (6, 17.0, 3.8, [0.65, 0.76, 0.9]),
        ] {
            let planet = &sim.bodies[host];
            let radial = DVec2::new(angle.cos(), angle.sin());
            let speed = (GRAVITY * planet.mass * radius.powi(2)
                / (radius.powi(2) + SOFTENING.powi(2)).powf(1.5))
            .sqrt();
            let position = planet.motion.position + radial * radius;
            let velocity = planet.motion.velocity + DVec2::new(radial.y, -radial.x) * speed;
            sim.bodies.push(Body::new(
                position,
                velocity,
                0.02,
                Rgba::new(rgb[0], rgb[1], rgb[2], 1.0),
            ));
        }
        let total_mass: f64 = sim.bodies.iter().map(|b| b.mass).sum();
        let drift: DVec2 = sim
            .bodies
            .iter()
            .map(|b| b.motion.velocity * b.mass)
            .sum::<DVec2>()
            / total_mass;
        for body in &mut sim.bodies {
            body.motion.value.velocity -= drift;
        }
        sim
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn add_body(&mut self, position: DVec2, spec: BodySpec) -> Result<(), &'static str> {
        spec.validate()?;
        if !position.is_finite() || position.abs().max_element() > 1e8 {
            return Err("Position is outside the supported world");
        }
        // Bound the quadratic pair solver, including repeated touch/click input.
        if self.bodies.len() >= 256 {
            return Err("Body limit reached (256)");
        }
        let mut body = Body::new(position, spec.velocity, spec.mass, spec.color);
        body.trail = crate::trail::Trail::at_step(position, self.steps);
        self.bodies.push(body);
        Ok(())
    }

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
            gravity: GRAVITY,
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
                let acceleration =
                    delta * self.gravity / (delta.length_squared() + SOFTENING.powi(2)).powf(1.5);
                body.motion.deriv.velocity += acceleration * other.mass;
                other.motion.deriv.velocity -= acceleration * body.mass;
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
