use super::{Body, Motion, Rk4, Simulation, Solver};
use crate::trail::{RetiredTrail, Trail};
use wgame::{glam::DVec2, rgb::Rgba};

impl Simulation {
    pub(super) fn step_with_collisions(&mut self, dt: f32) {
        let mut remaining = dt;
        loop {
            let before: Vec<Motion> = self.bodies.iter().map(|b| b.motion.value).collect();
            if remaining > 0.0 {
                Rk4.solve_step(self, remaining);
            }
            let mut first = None;
            for i in 0..self.bodies.len() {
                for j in 0..i {
                    let start = before[i].position - before[j].position;
                    let end = self.bodies[i].motion.position - self.bodies[j].motion.position;
                    if let Some(time) =
                        contact_time(start, end, self.bodies[i].radius + self.bodies[j].radius)
                        && first.is_none_or(|(earliest, _, _)| time < earliest)
                    {
                        first = Some((time, i, j));
                    }
                }
            }
            let Some((time, i, j)) = first else {
                break;
            };
            // Linear sweeps approximate motion within a 1/240-second RK4 step.
            // Rewind to the first contact, then integrate the remaining time
            // with the new masses. This avoids fast bodies tunnelling through
            // each other or colliding along paths they no longer follow.
            for (body, old) in self.bodies.iter_mut().zip(before) {
                body.motion.value = Motion {
                    position: old.position.lerp(body.motion.position, time),
                    velocity: old.velocity.lerp(body.motion.velocity, time),
                };
            }
            self.merge_contacts(i, j);
            remaining *= (1.0 - time) as f32;
            // Every pass removes at least one body, including zero-time
            // overlaps. Also resolve overlaps caused by a merged body's size.
        }
    }

    fn merge_contacts(&mut self, first: usize, second: usize) {
        let mut groups: Vec<usize> = (0..self.bodies.len()).collect();
        for i in 0..self.bodies.len() {
            for j in 0..i {
                let radius = self.bodies[i].radius + self.bodies[j].radius;
                let touching = self.bodies[i]
                    .motion
                    .position
                    .distance(self.bodies[j].motion.position)
                    <= radius + 1e-8 * radius.max(1.0);
                if touching || (i == first && j == second) {
                    let from = groups[i];
                    let to = groups[j];
                    for group in &mut groups {
                        if *group == from {
                            *group = to;
                        }
                    }
                }
            }
        }
        let mut members: Vec<Vec<Body>> = (0..self.bodies.len()).map(|_| Vec::new()).collect();
        for (body, group) in std::mem::take(&mut self.bodies).into_iter().zip(groups) {
            members[group].push(body);
        }
        for mut group in members {
            if group.is_empty() {
                continue;
            }
            if group.len() == 1 {
                self.bodies.push(group.pop().unwrap());
                continue;
            }
            let mass: f64 = group.iter().map(|b| b.mass).sum();
            let position = group
                .iter()
                .map(|b| b.motion.position * b.mass)
                .sum::<DVec2>()
                / mass;
            let velocity = group
                .iter()
                .map(|b| b.motion.velocity * b.mass)
                .sum::<DVec2>()
                / mass;
            let channel = |get: fn(Rgba<f32>) -> f32| {
                (group
                    .iter()
                    .map(|b| f64::from(get(b.color)) * b.mass)
                    .sum::<f64>()
                    / mass) as f32
            };
            let color = Rgba::new(
                channel(|c| c.r),
                channel(|c| c.g),
                channel(|c| c.b),
                channel(|c| c.a),
            );
            let mut merged = Body::new(position, velocity, mass, color);
            // Old trails describe separate trajectories; begin a fresh trail
            // instead of drawing a spurious connection to one parent's past.
            merged.trail = Trail::at_step(position, self.steps + 1);
            for body in group {
                self.retired_trails.push(RetiredTrail {
                    trail: body.trail,
                    position: body.motion.position,
                    radius: body.radius,
                    color: body.color,
                    step: self.steps + 1,
                });
            }
            self.bodies.push(merged);
        }
    }
}

/// First intersection of a relative-motion segment with a contact circle.
fn contact_time(start: DVec2, end: DVec2, radius: f64) -> Option<f64> {
    let c = start.length_squared() - radius * radius;
    if c <= 0.0 {
        return Some(0.0);
    }
    let delta = end - start;
    let a = delta.length_squared();
    let b = start.dot(delta);
    if a == 0.0 || b >= 0.0 {
        return None;
    }
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    // Stable form of the smaller quadratic root, including grazing contact.
    let time = c / (-b + discriminant.sqrt());
    (time <= 1.0).then_some(time)
}

#[cfg(test)]
mod tests;
