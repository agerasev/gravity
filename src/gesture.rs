use crate::{timing::Clock, viewport::Viewport};
use gravity::{BodySpec, Prediction, Simulation, body_radius};
use std::time::Duration;
use wgame::{
    Library,
    gfx::Scene,
    glam::{DVec2, Vec2},
    prelude::*,
};

pub const FORECAST_SPEED: u32 = 12;
pub const AIM_THRESHOLD: f32 = 6.0;
const DRAG_VELOCITY_SCALE: f64 = 0.2;

pub enum Gesture {
    Pan {
        last: Vec2,
    },
    Launch {
        accepted_pixel: Vec2,
        position: DVec2,
        spec: BodySpec,
        prediction: Option<Box<Prediction>>,
        forecast_clock: Clock,
    },
}
impl Gesture {
    pub fn launch(pixel: Vec2, position: DVec2, spec: BodySpec) -> Self {
        Self::Launch {
            accepted_pixel: pixel,
            position,
            spec,
            prediction: None,
            forecast_clock: Clock::default(),
        }
    }

    /// Run more fixed physics steps per wall-clock tick for a faster forecast.
    /// Keep the live solver's accuracy and cap wall-time catch-up after stalls.
    pub fn advance_prediction(
        &mut self,
        source: &Simulation,
        elapsed: Duration,
        running: bool,
    ) -> Result<(), &'static str> {
        if let Self::Launch {
            position,
            spec,
            prediction,
            forecast_clock,
            ..
        } = self
        {
            if prediction.is_none() {
                *prediction = Some(Box::new(Prediction::new(source, *position, *spec)?));
                forecast_clock.reset();
            } else {
                for _ in 0..forecast_clock.advance(elapsed, running) * FORECAST_SPEED {
                    prediction.as_mut().unwrap().step();
                }
            }
        }
        Ok(())
    }

    pub fn forecast_time(&self) -> Option<f64> {
        if let Self::Launch {
            prediction: Some(prediction),
            ..
        } = self
        {
            Some(prediction.time())
        } else {
            None
        }
    }

    pub fn is_launch(&self) -> bool {
        matches!(self, Self::Launch { .. })
    }
    pub fn update(&mut self, point: Vec2, view: &mut Viewport, size: Vec2) -> Option<DVec2> {
        match self {
            Self::Pan { last } => {
                view.pan(point - *last);
                *last = point;
                None
            }
            Self::Launch {
                accepted_pixel,
                position,
                spec,
                prediction,
                ..
            } => {
                // Compare against the last accepted aim, so slow deliberate
                // movement accumulates while pointer jitter leaves the shot
                // and its forecast untouched, including on button release.
                if point.distance(*accepted_pixel) >= AIM_THRESHOLD {
                    *accepted_pixel = point;
                    let velocity = ((view.world(point, size) - *position) * DRAG_VELOCITY_SCALE)
                        .clamp(DVec2::splat(-10_000.0), DVec2::splat(10_000.0));
                    let velocity = (velocity * 100.0).round() / 100.0;
                    if velocity != spec.velocity {
                        spec.velocity = velocity;
                        *prediction = None;
                    }
                    Some(spec.velocity)
                } else {
                    None
                }
            }
        }
    }
    pub fn draw(&self, library: &Library, scene: &mut Scene, zoom: f64) {
        if let Self::Launch {
            position,
            spec,
            prediction,
            ..
        } = self
        {
            let shapes = library.shapes();
            let start = position.as_vec2();
            let mut color = spec.color;
            color.a = 0.6;
            scene.add(
                &shapes
                    .unit_circle()
                    .scale(body_radius(spec.mass) as f32)
                    .move_to(start)
                    .fill_color(color),
            );
            if let Some(prediction) = prediction {
                let points: Vec<_> = prediction
                    .points()
                    .map(|p| wgame::shapes::PolylinePoint {
                        position: p.as_vec2(),
                        width: 1.5 / zoom as f32,
                    })
                    .collect();
                scene.add(&shapes.polyline(&points).fill_color(spec.color));
                let direction = prediction.velocity().as_vec2().try_normalize().or_else(|| {
                    points
                        .windows(2)
                        .rev()
                        .find_map(|pair| (pair[1].position - pair[0].position).try_normalize())
                });
                if let Some(direction) = direction {
                    let end = prediction.position().as_vec2();
                    let tip_size = (10.0 / zoom as f32).min(prediction.length() as f32 * 0.3);
                    if tip_size > 0.001 {
                        let back = end - direction * tip_size;
                        let normal = Vec2::new(-direction.y, direction.x) * tip_size * 0.45;
                        scene.add(
                            &shapes
                                .triangle(end, back + normal, back - normal)
                                .fill_color(prediction.color()),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgame::rgb::Rgba;
    #[test]
    fn click_preserves_exact_velocity_and_drag_uses_world_units() {
        let size = Vec2::new(800.0, 600.0);
        let mut view = Viewport {
            center: DVec2::new(10.0, 20.0),
            zoom: 2.0,
        };
        let pixel = Vec2::new(200.0, 200.0);
        let spec = BodySpec {
            mass: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            velocity: DVec2::new(3.25, -2.1),
        };
        let mut drag = Gesture::launch(pixel, view.world(pixel, size), spec);
        assert_eq!(drag.update(pixel + Vec2::ONE, &mut view, size), None);
        if let Gesture::Launch { spec: current, .. } = drag {
            assert_eq!(current.velocity, spec.velocity);
        }
        assert_eq!(
            drag.update(pixel + Vec2::new(100.0, -80.0), &mut view, size),
            Some(DVec2::new(10.0, -8.0))
        );
    }

    #[test]
    fn slow_motion_accumulates_until_threshold_and_release_keeps_accepted_velocity() {
        let size = Vec2::new(800.0, 600.0);
        let pixel = size * 0.5;
        let mut view = Viewport::default();
        let spec = BodySpec {
            mass: 1.0,
            color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            velocity: DVec2::ZERO,
        };
        let mut drag = Gesture::launch(pixel, DVec2::ZERO, spec);
        for offset in 1..6 {
            assert_eq!(
                drag.update(pixel + Vec2::new(offset as f32, 0.0), &mut view, size),
                None
            );
        }
        assert_eq!(
            drag.update(pixel + Vec2::new(6.0, 0.0), &mut view, size),
            Some(DVec2::new(1.2, 0.0))
        );
        for offset in [7.0, 8.0, 7.5, 8.5] {
            assert_eq!(
                drag.update(pixel + Vec2::new(offset, 0.0), &mut view, size),
                None
            );
        }
        if let Gesture::Launch { spec, .. } = &drag {
            assert_eq!(spec.velocity, DVec2::new(1.2, 0.0));
        }
        assert_eq!(
            drag.update(pixel + Vec2::new(12.0, 0.0), &mut view, size),
            Some(DVec2::new(2.4, 0.0))
        );
    }
}

#[cfg(test)]
mod prediction_tests {
    use super::*;
    use wgame::rgb::Rgba;

    #[test]
    fn forecast_runs_faster_and_keeps_growing_through_pointer_jitter() {
        let source = Simulation::solar_system();
        let mut view = Viewport::default();
        let size = Vec2::new(800.0, 600.0);
        let pixel = Vec2::new(700.0, 500.0);
        let spec = BodySpec {
            mass: 1.0,
            color: Rgba::new(1.0, 0.5, 0.0, 1.0),
            velocity: DVec2::new(10.0, 20.0),
        };
        let mut gesture = Gesture::launch(pixel, view.world(pixel, size), spec);
        let frame = Duration::from_millis(10);
        gesture.advance_prediction(&source, frame, true).unwrap();
        assert_eq!(gesture.forecast_time(), Some(0.0));
        let mut live_clock = Clock::default();
        let mut live_steps = 0;
        for _ in 0..100 {
            live_steps += live_clock.advance(frame, true);
            gesture.advance_prediction(&source, frame, true).unwrap();
        }
        assert_eq!(
            gesture.forecast_time(),
            Some(f64::from(live_steps * FORECAST_SPEED) / f64::from(gravity::STEPS_PER_SECOND))
        );
        gesture.update(pixel + Vec2::new(30.0, 40.0), &mut view, size);
        gesture.advance_prediction(&source, frame, true).unwrap();
        assert_eq!(gesture.forecast_time(), Some(0.0));
        gesture.advance_prediction(&source, frame, true).unwrap();
        let before = gesture.forecast_time().unwrap();
        // Small movement must preserve both the forecast and launch velocity.
        assert_eq!(
            gesture.update(pixel + Vec2::new(33.0, 42.0), &mut view, size),
            None
        );
        if let Gesture::Launch { spec, .. } = &gesture {
            assert_eq!(spec.velocity, DVec2::new(6.0, 8.0));
        }
        gesture.advance_prediction(&source, frame, true).unwrap();
        assert!(gesture.forecast_time().unwrap() > before);
        let before = gesture.forecast_time();
        gesture
            .advance_prediction(&source, Duration::from_secs(10), false)
            .unwrap();
        assert_eq!(gesture.forecast_time(), before);
        gesture.advance_prediction(&source, frame, true).unwrap();
        assert!(
            gesture.forecast_time().unwrap() - before.unwrap() < 0.02 * f64::from(FORECAST_SPEED)
        );
    }
}
