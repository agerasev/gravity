use crate::viewport::Viewport;
use gravity::{BodySpec, body_radius};
use wgame::{
    Library,
    gfx::Scene,
    glam::{DVec2, Vec2},
    prelude::*,
};

pub enum Gesture {
    Pan {
        last: Vec2,
    },
    Launch {
        pixel: Vec2,
        position: DVec2,
        spec: BodySpec,
        aimed: bool,
    },
}
impl Gesture {
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
                pixel,
                position,
                spec,
                aimed,
            } => {
                if point.distance(*pixel) > 5.0 {
                    *aimed = true;
                }
                if *aimed {
                    let velocity = (view.world(point, size) - *position)
                        .clamp(DVec2::splat(-10_000.0), DVec2::splat(10_000.0));
                    spec.velocity = (velocity * 100.0).round() / 100.0;
                    Some(spec.velocity)
                } else {
                    None
                }
            }
        }
    }
    pub fn draw(&self, library: &Library, scene: &mut Scene, zoom: f64) {
        if let Self::Launch { position, spec, .. } = self {
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
            let end = (*position + spec.velocity).as_vec2();
            let delta = end - start;
            if delta.length() > 0.001 {
                let direction = delta.normalize();
                let tip_size = (10.0 / zoom as f32).min(delta.length() * 0.3);
                let back = end - direction * tip_size;
                let normal = Vec2::new(-direction.y, direction.x) * tip_size * 0.45;
                scene.add(
                    &shapes
                        .line(start, end, 1.5 / zoom as f32)
                        .fill_color(spec.color),
                );
                scene.add(
                    &shapes
                        .triangle(end, back + normal, back - normal)
                        .fill_color(spec.color),
                );
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
        let mut drag = Gesture::Launch {
            pixel,
            position: view.world(pixel, size),
            spec,
            aimed: false,
        };
        assert_eq!(drag.update(pixel + Vec2::ONE, &mut view, size), None);
        if let Gesture::Launch { spec: current, .. } = drag {
            assert_eq!(current.velocity, spec.velocity);
        }
        assert_eq!(
            drag.update(pixel + Vec2::new(100.0, -80.0), &mut view, size),
            Some(DVec2::new(50.0, -40.0))
        );
    }
}
