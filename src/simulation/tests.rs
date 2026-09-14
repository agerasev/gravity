use super::*;

fn pair(positions: [DVec2; 2]) -> Simulation {
    Simulation {
        bodies: positions
            .map(|p| Body::new(p, DVec2::ZERO, 10.0, Rgba::new(1.0, 1.0, 1.0, 1.0)))
            .into(),
        steps: 0,
        gravity: 1e5,
    }
}

#[test]
fn force_law_and_pair_symmetry_match_the_original() {
    let mut sim = pair([DVec2::new(-100.0, 0.0), DVec2::new(100.0, 0.0)]);
    sim.compute_gravity();
    let expected = 1e5 / (200.0_f64.powi(2) + 120.0_f64.powi(2));
    assert!((sim.bodies[0].motion.deriv.velocity.x - expected).abs() < 1e-12);
    assert_eq!(
        sim.bodies[0].motion.deriv.velocity,
        -sim.bodies[1].motion.deriv.velocity
    );
    for _ in 0..240 {
        sim.step();
    }
    assert!((sim.bodies[0].motion.position + sim.bodies[1].motion.position).length() < 1e-10);
    assert!((sim.bodies[0].motion.velocity + sim.bodies[1].motion.velocity).length() < 1e-10);
    assert!(sim.bodies[0].motion.position.x > -100.0);
}

#[test]
fn coincident_bodies_stay_finite_without_an_invented_direction() {
    let mut sim = pair([DVec2::ZERO; 2]);
    for _ in 0..48 {
        sim.step();
    }
    for body in sim.bodies {
        assert_eq!(body.motion.position, DVec2::ZERO);
        assert_eq!(body.motion.velocity, DVec2::ZERO);
    }
}

#[test]
fn phy_parameter_retains_motion_below_f32_position_precision() {
    let mut sim = Simulation {
        bodies: vec![Body::new(
            DVec2::splat(1e9),
            DVec2::X,
            10.0,
            Rgba::new(1.0, 1.0, 1.0, 1.0),
        )],
        steps: 0,
        gravity: 1e5,
    };
    for _ in 0..240 {
        sim.step();
    }
    assert!((sim.bodies[0].motion.position.x - (1e9 + 1.0)).abs() < 1e-4);
    assert_eq!(sim.bodies[0].motion.position.y, 1e9);
}

#[test]
fn seeded_simulation_is_reproducible_and_stays_finite_with_full_trails() {
    let mut sim = Simulation::new(42);
    let same = Simulation::new(42);
    assert_eq!(sim.bodies.len(), 64);
    for (a, b) in sim.bodies.iter().zip(&same.bodies) {
        assert_eq!(a.motion.value, b.motion.value);
        assert_eq!(a.color, b.color);
    }
    for _ in 0..STEPS_PER_SECOND * 8 {
        sim.step();
    }
    let mut points = Vec::new();
    for body in &sim.bodies {
        assert!(body.motion.position.is_finite());
        assert!(body.motion.velocity.is_finite());
        body.trail
            .write_points(body.motion.position, sim.steps, body.radius, &mut points);
        assert!(points.len() <= 34);
        assert_eq!(points.last().unwrap().width, 0.0);
        assert!(
            points
                .iter()
                .all(|p| p.position.is_finite() && p.width.is_finite())
        );
    }
    assert_eq!(sim.time(), 8.0);
}
