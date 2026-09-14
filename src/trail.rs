use std::collections::VecDeque;

use wgame::{glam::DVec2, rgb::Rgba, shapes::PolylinePoint};

use crate::STEPS_PER_SECOND;

const SAMPLE_STEPS: u64 = STEPS_PER_SECOND as u64 / 5;
const SEGMENTS: usize = 32;
const LIFETIME_STEPS: u64 = SAMPLE_STEPS * SEGMENTS as u64;

#[derive(Clone, Copy)]
struct Sample {
    position: DVec2,
    step: u64,
}

pub(crate) struct Trail {
    samples: VecDeque<Sample>,
}

pub(crate) struct RetiredTrail {
    pub trail: Trail,
    pub position: DVec2,
    pub radius: f64,
    pub color: Rgba<f32>,
    pub step: u64,
}

impl RetiredTrail {
    pub fn is_alive(&self, step: u64) -> bool {
        step.saturating_sub(self.step) < LIFETIME_STEPS
    }

    pub fn write_points(&self, step: u64, points: &mut Vec<PolylinePoint>) {
        self.trail
            .write_aged_points(self.position, self.step, step, self.radius, points);
    }
}

impl Trail {
    pub fn new(position: DVec2) -> Self {
        Self::at_step(position, 0)
    }

    pub fn at_step(position: DVec2, step: u64) -> Self {
        let mut samples = VecDeque::with_capacity(SEGMENTS + 1);
        samples.push_back(Sample { position, step });
        Self { samples }
    }

    pub fn record(&mut self, position: DVec2, step: u64) {
        if step.is_multiple_of(SAMPLE_STEPS) {
            self.samples.push_back(Sample { position, step });
            if self.samples.len() > SEGMENTS + 1 {
                self.samples.pop_front();
            }
        }
    }

    /// Write newest-to-oldest junctions. Keep a sample on either side of the
    /// lifetime cutoff so the tail moves continuously between sampling ticks.
    pub fn write_points(
        &self,
        position: DVec2,
        step: u64,
        radius: f64,
        points: &mut Vec<PolylinePoint>,
    ) {
        self.write_aged_points(position, step, step, radius, points);
    }

    fn write_aged_points(
        &self,
        position: DVec2,
        head_step: u64,
        step: u64,
        radius: f64,
        points: &mut Vec<PolylinePoint>,
    ) {
        points.clear();
        let head_age = step.saturating_sub(head_step) as f64 / LIFETIME_STEPS as f64;
        if head_age >= 1.0 {
            return;
        }
        points.push(PolylinePoint {
            position: position.as_vec2(),
            width: (2.0 * radius * (1.0 - head_age)) as f32,
        });
        let cutoff = step.saturating_sub(LIFETIME_STEPS);
        let mut newer = Sample {
            position,
            step: head_step,
        };
        for sample in self
            .samples
            .iter()
            .rev()
            .filter(|sample| sample.step < head_step)
        {
            if sample.step < cutoff {
                let fraction = (newer.step - cutoff) as f64 / (newer.step - sample.step) as f64;
                points.push(PolylinePoint {
                    position: newer.position.lerp(sample.position, fraction).as_vec2(),
                    width: 0.0,
                });
                break;
            }
            let age = (step - sample.step) as f64 / LIFETIME_STEPS as f64;
            points.push(PolylinePoint {
                position: sample.position.as_vec2(),
                width: (2.0 * radius * (1.0 - age)) as f32,
            });
            newer = *sample;
        }
    }
}

#[cfg(test)]
mod tests;
