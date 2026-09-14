//! A softened gravitational particle simulation with time-limited trails.
#![forbid(unsafe_code)]

mod body;
mod render;
mod simulation;
mod trail;

pub use simulation::{BodySpec, STEPS_PER_SECOND, SYSTEM_VIEW_SIZE, Simulation, body_radius};
