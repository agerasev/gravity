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
fn softened_force_and_pair_symmetry() {
    let mut sim = pair([DVec2::new(-100.0, 0.0), DVec2::new(100.0, 0.0)]);
    sim.compute_gravity();
    let expected = 1e5 * 10.0 * 200.0 / (200.0_f64.powi(2) + SOFTENING.powi(2)).powf(1.5);
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

#[test]
fn unequal_masses_conserve_momentum() {
    let mut sim = pair([DVec2::new(-100.0, 0.0), DVec2::new(100.0, 0.0)]);
    sim.bodies[1].mass = 30.0;
    sim.compute_gravity();
    assert!(
        (sim.bodies[0].motion.deriv.velocity + sim.bodies[1].motion.deriv.velocity * 3.0).length()
            < 1e-10
    );
    for _ in 0..240 {
        sim.step();
    }
    let momentum: DVec2 = sim.bodies.iter().map(|b| b.motion.velocity * b.mass).sum();
    assert!(momentum.length() < 1e-8);
}

#[test]
fn solar_orbits_and_moons_remain_bound_for_three_minutes() {
    let mut sim = Simulation::solar_system();
    assert_eq!(sim.body_count(), 10);
    let radii: Vec<f64> = sim.bodies[1..8]
        .iter()
        .map(|b| b.motion.position.length())
        .collect();
    for _ in 0..STEPS_PER_SECOND * 180 {
        sim.step();
        for (i, radius) in radii.iter().enumerate() {
            let distance = sim.bodies[i + 1]
                .motion
                .position
                .distance(sim.bodies[0].motion.position);
            assert!(
                (radius * 0.7..radius * 1.3).contains(&distance),
                "planet {i}: {distance} vs {radius}"
            );
        }
        for (moon, host) in [(8, 5), (9, 6)] {
            let distance = sim.bodies[moon]
                .motion
                .position
                .distance(sim.bodies[host].motion.position);
            assert!((8.0..30.0).contains(&distance), "moon {moon}: {distance}");
        }
    }
}

#[test]
fn inserted_body_has_exact_properties_and_no_history_before_birth() {
    let mut sim = Simulation::solar_system();
    for _ in 0..STEPS_PER_SECOND * 8 {
        sim.step();
    }
    let spec = BodySpec {
        mass: 12.5,
        color: Rgba::new(0.4, 0.8, 1.0, 1.0),
        velocity: DVec2::new(-3.25, 41.75),
    };
    let position = DVec2::new(600.0, 300.0);
    sim.add_body(position, spec).unwrap();
    let body = sim.bodies.last().unwrap();
    assert_eq!(body.mass, spec.mass);
    assert_eq!(body.color, spec.color);
    assert_eq!(body.motion.position, position);
    assert_eq!(body.motion.velocity, spec.velocity);
    let mut points = Vec::new();
    body.trail
        .write_points(position, sim.steps, body.radius, &mut points);
    assert_eq!(points.len(), 1);
    sim.step();
    let body = sim.bodies.last().unwrap();
    body.trail
        .write_points(body.motion.position, sim.steps, body.radius, &mut points);
    assert_eq!(points.len(), 2);
    assert!(points[1].width > points[0].width * 0.99);
    let count = sim.body_count();
    assert!(
        sim.add_body(
            position,
            BodySpec {
                mass: f64::NAN,
                ..spec
            }
        )
        .is_err()
    );
    assert!(sim.add_body(DVec2::splat(f64::INFINITY), spec).is_err());
    assert_eq!(sim.body_count(), count);
}
