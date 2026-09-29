use super::{BodySpec, STEPS_PER_SECOND, Simulation};
use wgame::{glam::DVec2, rgb::Rgba};

const MAX_SAMPLES: usize = 4096;

/// An isolated simulation of a proposed launch. Each step advances the entire
/// system with the live solver and collision rules. After an impact, tracking
/// continues with the merged body containing the launch's mass.
pub struct Prediction {
    simulation: Simulation,
    steps: u64,
    samples: Vec<DVec2>,
    sample_stride: u64,
    position: DVec2,
    velocity: DVec2,
    color: Rgba<f32>,
    length: f64,
}

impl Prediction {
    pub fn new(source: &Simulation, position: DVec2, spec: BodySpec) -> Result<Self, &'static str> {
        let mut simulation = source.fork_physics();
        simulation.add_body(position, spec)?;
        simulation.bodies.last_mut().unwrap().tracked = true;
        Ok(Self {
            simulation,
            steps: 0,
            samples: vec![position],
            sample_stride: 4,
            position,
            velocity: spec.velocity,
            color: spec.color,
            length: 0.0,
        })
    }

    pub fn step(&mut self) {
        self.simulation.step();
        self.steps += 1;
        let body = self
            .simulation
            .bodies
            .iter()
            .find(|body| body.tracked)
            .expect("merging preserves the tracked body");
        self.length += self.position.distance(body.motion.position);
        self.position = body.motion.position;
        self.velocity = body.motion.velocity;
        self.color = body.color;
        if self.steps.is_multiple_of(self.sample_stride) {
            self.samples.push(self.position);
            if self.samples.len() > MAX_SAMPLES {
                // Keep the entire horizon, at progressively coarser drawing
                // resolution. Physics always retains the same fixed time step.
                let mut index = 0;
                self.samples.retain(|_| {
                    let keep = index % 2 == 0;
                    index += 1;
                    keep
                });
                self.sample_stride *= 2;
            }
        }
    }

    pub fn time(&self) -> f64 {
        self.steps as f64 / f64::from(STEPS_PER_SECOND)
    }
    pub fn position(&self) -> DVec2 {
        self.position
    }
    pub fn velocity(&self) -> DVec2 {
        self.velocity
    }
    pub fn color(&self) -> Rgba<f32> {
        self.color
    }
    pub fn length(&self) -> f64 {
        self.length
    }

    /// Sampled path plus the latest position, even between drawing samples.
    pub fn points(&self) -> impl Iterator<Item = DVec2> + '_ {
        self.samples
            .iter()
            .copied()
            .chain(std::iter::once(self.position))
    }
}

#[cfg(test)]
mod tests;
