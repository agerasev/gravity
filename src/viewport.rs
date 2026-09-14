use wgame::glam::{Affine2, DVec2, Vec2};

pub struct Viewport {
    pub center: DVec2,
    pub zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center: DVec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub fn home(&mut self, size: Vec2) {
        self.center = DVec2::ZERO;
        self.zoom = (f64::from(size.min_element()) / 1350.0).clamp(0.02, 30.0);
    }

    pub fn world(&self, pixel: Vec2, size: Vec2) -> DVec2 {
        self.center + (pixel - size * 0.5).as_dvec2() / self.zoom
    }

    pub fn transform(&self, size: Vec2) -> Affine2 {
        Affine2::from_scale_angle_translation(
            Vec2::splat(self.zoom as f32),
            0.0,
            size * 0.5 - (self.center * self.zoom).as_vec2(),
        )
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.center = (self.center - delta.as_dvec2() / self.zoom)
            .clamp(DVec2::splat(-1e8), DVec2::splat(1e8));
    }

    pub fn zoom_at(&mut self, factor: f64, pixel: Vec2, size: Vec2) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let anchor = self.world(pixel, size);
        self.zoom = (self.zoom * factor).clamp(0.02, 30.0);
        self.center += anchor - self.world(pixel, size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_preserves_cursor_anchor_even_at_limits() {
        let size = Vec2::new(900.0, 700.0);
        let cursor = Vec2::new(126.0, 567.0);
        let mut view = Viewport::default();
        view.home(size);
        view.pan(Vec2::new(130.0, -91.0));
        let world = view.world(cursor, size);
        for factor in [2.0, 0.5, 1e6, 1e-9] {
            view.zoom_at(factor, cursor, size);
            assert!((view.world(cursor, size) - world).length() < 1e-9);
            let screen = view.transform(size).transform_point2(world.as_vec2());
            assert!((screen - cursor).length() < 0.05);
        }
    }
    #[test]
    fn pan_tracks_pointer_and_home_restores_system() {
        let size = Vec2::new(800.0, 600.0);
        let mut view = Viewport::default();
        view.home(size);
        let before = view.world(Vec2::new(10.0, 20.0), size);
        view.pan(Vec2::new(75.0, 100.0));
        assert!((view.world(Vec2::new(85.0, 120.0), size) - before).length() < 1e-9);
        view.home(size);
        assert_eq!(view.world(size * 0.5, size), DVec2::ZERO);
    }
}
