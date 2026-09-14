use std::ops::{AddAssign, MulAssign};

use phy::{Deriv, Param, Rk4, Var};
use wgame::{glam::DVec2, rgb::Rgba};

use crate::trail::Trail;

/// Keep state and derivative accumulation in f64. phy supplies f32 time steps
/// and RK4 coefficients, which are converted at the parameter boundary.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct Motion {
    pub position: DVec2,
    pub velocity: DVec2,
}

impl Param for Motion {
    type Deriv = Self;

    fn step(&mut self, deriv: &Self, dt: f32) {
        self.position += deriv.position * f64::from(dt);
        self.velocity += deriv.velocity * f64::from(dt);
    }
}

impl Deriv for Motion {}

impl MulAssign<f32> for Motion {
    fn mul_assign(&mut self, factor: f32) {
        self.position *= f64::from(factor);
        self.velocity *= f64::from(factor);
    }
}

impl AddAssign<&Self> for Motion {
    fn add_assign(&mut self, other: &Self) {
        self.position += other.position;
        self.velocity += other.velocity;
    }
}

pub(crate) struct Body {
    pub motion: Var<Motion, Rk4>,
    pub mass: f64,
    pub radius: f64,
    pub color: Rgba<f32>,
    pub trail: Trail,
}

impl Body {
    pub fn new(position: DVec2, velocity: DVec2, mass: f64, color: Rgba<f32>) -> Self {
        Self {
            motion: Var::new(Motion { position, velocity }),
            mass,
            radius: mass,
            color,
            trail: Trail::new(position),
        }
    }
}
