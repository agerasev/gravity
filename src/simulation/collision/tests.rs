use super::*;

fn simulation(bodies: Vec<Body>) -> Simulation {
    Simulation {
        bodies,
        steps: 0,
        retired_trails: Vec::new(),
        gravity: 0.0,
        collisions: true,
    }
}
fn body(x: f64, mass: f64, vx: f64, color: Rgba<f32>) -> Body {
    Body::new(DVec2::new(x, 0.0), DVec2::new(vx, 0.0), mass, color)
}
const RED: Rgba<f32> = Rgba::new(1.0, 0.0, 0.0, 1.0);
const BLUE: Rgba<f32> = Rgba::new(0.0, 0.0, 1.0, 0.5);

#[test]
fn merge_conserves_mass_momentum_center_and_weighted_color() {
    let mut sim = simulation(vec![body(-1.0, 2.0, 12.0, RED), body(1.0, 6.0, -4.0, BLUE)]);
    sim.steps = 2000;
    sim.step();
    assert_eq!(sim.body_count(), 1);
    let merged = &sim.bodies[0];
    assert_eq!(merged.mass, 8.0);
    assert_eq!(merged.motion.velocity, DVec2::ZERO);
    assert_eq!(merged.motion.position, DVec2::new(0.5, 0.0));
    assert_eq!(merged.color, Rgba::new(0.25, 0.0, 0.75, 0.625));
    assert_eq!(merged.radius, crate::body_radius(8.0));
    let mut points = Vec::new();
    merged.trail.write_points(
        merged.motion.position,
        sim.steps,
        merged.radius,
        &mut points,
    );
    assert_eq!(points.len(), 1);
}

#[test]
fn simultaneous_contact_chain_merges_as_one_mass_weighted_group() {
    let mut sim = simulation(vec![
        body(-8.0, 1.0, 0.0, RED),
        body(0.0, 1.0, 0.0, BLUE),
        body(8.0, 1.0, 0.0, Rgba::new(0.0, 1.0, 0.0, 1.0)),
    ]);
    sim.step();
    assert_eq!(sim.body_count(), 1);
    let merged = &sim.bodies[0];
    assert_eq!(merged.mass, 3.0);
    assert_eq!(merged.motion.position, DVec2::ZERO);
    assert_eq!(
        [merged.color.r, merged.color.g, merged.color.b],
        [1.0 / 3.0; 3]
    );
}

#[test]
fn fast_crossing_merges_but_near_miss_does_not() {
    let mut sim = simulation(vec![
        body(-20.0, 1.0, 10000.0, RED),
        body(20.0, 1.0, -10000.0, BLUE),
    ]);
    sim.step();
    assert_eq!(sim.body_count(), 1);
    assert!(sim.bodies[0].motion.position.length() < 1e-10);
    assert_eq!(sim.bodies[0].motion.velocity, DVec2::ZERO);
    assert_eq!(
        contact_time(DVec2::new(-20.0, 10.0), DVec2::new(20.0, 10.0), 8.4),
        None
    );
    assert_eq!(
        contact_time(DVec2::new(-10.0, 2.0), DVec2::new(10.0, 2.0), 2.0),
        Some(0.5)
    );
}

#[test]
fn impact_changes_the_remaining_path_before_any_further_collision() {
    let mut sim = simulation(vec![
        body(-60.0, 1.0, 20000.0, RED),
        body(0.0, 1000.0, 0.0, BLUE),
        body(25.0, 1.0, 0.0, RED),
    ]);
    sim.step();
    assert_eq!(sim.body_count(), 2);
    let momentum: DVec2 = sim.bodies.iter().map(|b| b.motion.velocity * b.mass).sum();
    assert!((momentum.x - 20000.0).abs() < 1e-8);
}

#[test]
fn growing_body_can_merge_again_and_disabled_collisions_preserve_overlap() {
    let mut sim = simulation(vec![
        body(0.0, 1.0, 0.0, RED),
        body(0.0, 1.0, 0.0, BLUE),
        body(8.6, 1.0, 0.0, RED),
    ]);
    sim.set_collisions_enabled(false);
    sim.step();
    assert_eq!(sim.body_count(), 3);
    sim.set_collisions_enabled(true);
    assert_eq!(sim.body_count(), 3);
    sim.step();
    assert_eq!(sim.body_count(), 1);
    assert_eq!(sim.bodies[0].mass, 3.0);
    assert!((sim.bodies[0].color.r - 2.0 / 3.0).abs() < 1e-7);
}

#[test]
fn both_original_trails_survive_merge_then_expire() {
    let mut sim = simulation(vec![
        body(-20.0, 1.0, 20.0, RED),
        body(20.0, 1.0, -20.0, BLUE),
    ]);
    for _ in 0..240 {
        sim.step();
        if sim.body_count() == 1 {
            break;
        }
    }
    assert_eq!(sim.body_count(), 1);
    assert_eq!(sim.retired_trails.len(), 2);
    assert_eq!(sim.retired_trails[0].color, RED);
    assert_eq!(sim.retired_trails[1].color, BLUE);
    for trail in &sim.retired_trails {
        let mut points = Vec::new();
        trail.write_points(sim.steps, &mut points);
        assert!(points.len() >= 3);
    }
    for _ in 0..super::super::STEPS_PER_SECOND * 7 {
        sim.step();
    }
    assert!(sim.retired_trails.is_empty());
}
