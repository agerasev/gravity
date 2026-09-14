use std::collections::VecDeque;

use wgame::{glam::DVec2, shapes::PolylinePoint};

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

impl Trail {
    pub fn new(position: DVec2) -> Self {
        let mut samples = VecDeque::with_capacity(SEGMENTS + 1);
        samples.push_back(Sample { position, step: 0 });
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
        points.clear();
        points.push(PolylinePoint {
            position: position.as_vec2(),
            width: (2.0 * radius) as f32,
        });
        let cutoff = step.saturating_sub(LIFETIME_STEPS);
        let mut newer = Sample { position, step };
        for sample in self
            .samples
            .iter()
            .rev()
            .filter(|sample| sample.step < step)
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
