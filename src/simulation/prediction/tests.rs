use super::*;
use crate::body::Body;

fn spec() -> BodySpec {
    BodySpec {
        mass: 500.0,
        color: Rgba::new(0.2, 0.7, 1.0, 1.0),
        velocity: DVec2::new(-50.0, 25.0),
    }
}

#[test]
fn forecast_matches_actual_launch_without_changing_the_source() {
    for collisions in [false, true] {
        let mut source = Simulation::solar_system();
        source.set_collisions_enabled(collisions);
        for _ in 0..240 {
            source.step();
        }
        let original: Vec<_> = source.bodies.iter().map(|b| b.motion.value).collect();
        let mut actual = source.fork_physics();
        let position = DVec2::new(180.0, 50.0);
        let mut forecast = Prediction::new(&source, position, spec()).unwrap();
        actual.add_body(position, spec()).unwrap();
        for _ in 0..720 {
            forecast.step();
            actual.step();
            assert_eq!(forecast.simulation.body_count(), actual.body_count());
            for (a, b) in forecast.simulation.bodies.iter().zip(&actual.bodies) {
                assert_eq!(a.motion.value, b.motion.value);
                assert_eq!(a.mass, b.mass);
                assert_eq!(a.color, b.color);
            }
        }
        assert_eq!(forecast.time(), 3.0);
        assert_eq!(source.time(), 1.0);
        assert_eq!(
            source
                .bodies
                .iter()
                .map(|b| b.motion.value)
                .collect::<Vec<_>>(),
            original
        );
    }
}

#[test]
fn tracking_survives_other_merges_then_follows_its_own_merge() {
    let source = Simulation {
        bodies: vec![
            Body::new(DVec2::ZERO, DVec2::ZERO, 1.0, spec().color),
            Body::new(DVec2::ZERO, DVec2::ZERO, 2.0, spec().color),
        ],
        steps: 0,
        retired_trails: Vec::new(),
        gravity: 0.0,
        collisions: true,
    };
    let launch = BodySpec {
        mass: 1.0,
        velocity: DVec2::new(-100.0, 0.0),
        ..spec()
    };
    let mut forecast = Prediction::new(&source, DVec2::new(50.0, 0.0), launch).unwrap();
    forecast.step();
    assert_eq!(forecast.simulation.body_count(), 2);
    assert!(forecast.position().x > 49.0);
    for _ in 0..240 {
        forecast.step();
    }
    assert_eq!(forecast.simulation.body_count(), 1);
    assert_eq!(forecast.velocity(), DVec2::new(-25.0, 0.0));
    assert_eq!(
        forecast.position(),
        forecast.simulation.bodies[0].motion.position
    );
}

#[test]
fn long_forecasts_keep_the_start_and_exact_tip_with_bounded_samples() {
    let source = Simulation {
        bodies: Vec::new(),
        steps: 0,
        retired_trails: Vec::new(),
        gravity: 0.0,
        collisions: false,
    };
    let start = DVec2::new(10.0, 20.0);
    let mut forecast = Prediction::new(&source, start, spec()).unwrap();
    for _ in 0..40_000 {
        forecast.step();
    }
    assert!(forecast.samples.len() <= MAX_SAMPLES);
    assert_eq!(forecast.points().next().unwrap(), start);
    assert_eq!(forecast.points().last().unwrap(), forecast.position());
    assert!((forecast.position() - (start + spec().velocity * forecast.time())).length() < 0.001);
}
