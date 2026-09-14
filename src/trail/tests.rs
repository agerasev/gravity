use super::*;

#[test]
fn history_is_bounded_and_oldest_segment_is_clipped_continuously() {
    let mut trail = Trail::new(DVec2::ZERO);
    let mut points = Vec::new();
    for step in 1..=2000 {
        let position = DVec2::new(step as f64, 0.0);
        trail.record(position, step);
        trail.write_points(position, step, 10.0, &mut points);
        assert!(trail.samples.len() <= SEGMENTS + 1);
        assert!(points.len() <= SEGMENTS + 2);
        assert_eq!(points[0].position.x, step as f32);
        assert_eq!(points[0].width, 20.0);
        assert!(points.windows(2).all(|p| p[0].width >= p[1].width));
        let tail = points.last().unwrap();
        if step >= LIFETIME_STEPS {
            assert_eq!(tail.width, 0.0);
            assert_eq!(tail.position.x, (step - LIFETIME_STEPS) as f32);
        } else {
            assert_eq!(tail.position.x, 0.0);
        }
    }
}

#[test]
fn sampling_boundary_does_not_duplicate_the_current_junction() {
    let mut trail = Trail::new(DVec2::ZERO);
    let mut points = Vec::new();
    for step in 1..=SAMPLE_STEPS + 1 {
        let position = DVec2::new(step as f64, 0.0);
        trail.record(position, step);
        trail.write_points(position, step, 1.0, &mut points);
        if step <= SAMPLE_STEPS {
            assert_eq!(points.len(), 2);
        } else {
            assert_eq!(points.len(), 3);
        }
        assert!(points.windows(2).all(|p| p[0].position != p[1].position));
    }
}
