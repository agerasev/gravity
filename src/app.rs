use crate::{
    controls::{Action, Controls},
    gesture::Gesture,
    timing::Clock,
    viewport::Viewport,
};
use gravity::Simulation;
use wgame::{
    Event, Library, Result, Window,
    app::time::Instant,
    gfx::types::color,
    glam::Vec2,
    input::{
        event::{MouseButton, MouseScrollDelta, TouchPhase},
        keyboard::{KeyCode, PhysicalKey},
    },
    prelude::*,
};

#[derive(Clone, Copy, PartialEq)]
enum Pointer {
    Mouse(MouseButton),
    Touch(u64),
}

fn move_pointer(
    point: Vec2,
    gesture: &mut Option<(Pointer, Gesture)>,
    controls: &mut Controls,
    view: &mut Viewport,
    size: Vec2,
) {
    if let Some((_, gesture)) = gesture
        && let Some(velocity) = gesture.update(point, view, size)
    {
        controls.message = format!("Aim: {:.2}, {:.2} units/sec", velocity.x, velocity.y);
    }
}

fn release_pointer(
    pointer: Pointer,
    point: Vec2,
    gesture: &mut Option<(Pointer, Gesture)>,
    controls: &mut Controls,
    simulation: &mut Simulation,
    size: Vec2,
) {
    if gesture.as_ref().is_none_or(|(owner, _)| *owner != pointer) {
        return;
    }
    if let Some((
        _,
        Gesture::Launch {
            position,
            spec,
            aimed,
            ..
        },
    )) = gesture.take()
    {
        if controls.covers(point, size) {
            controls.message = "Launch cancelled over controls".into();
            return;
        }
        match simulation.add_body(position, spec) {
            Ok(()) => {
                if aimed {
                    controls.set_velocity(spec.velocity);
                }
                controls.message = "Body launched".into();
            }
            Err(error) => controls.message = error.into(),
        }
    }
}

pub async fn run(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let mut controls = Controls::new(&library)?;
    let mut simulation = Simulation::solar_system();
    let mut view = Viewport::default();
    let mut input = window.input();
    // Store physical cursor coordinates so display-scale changes do not stale it.
    let mut cursor = None;
    let mut scale_factor = window.scale_factor();
    let mut gesture: Option<(Pointer, Gesture)> = None;
    let mut focused = true;
    let mut clock = Clock::default();
    let mut last = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;

    'frames: while let Some(mut frame) = window.next_frame().await? {
        let logical_size = frame.logical_size();
        let size = Vec2::new(logical_size.0 as f32, logical_size.1 as f32);
        let scale_changed = frame.scale_factor() != scale_factor;
        scale_factor = frame.scale_factor();
        let mut world_size = controls.world_size(size);
        if frames == 0 {
            view.home(world_size);
        }
        let mut reset_clock = frame.resized().is_some() || scale_changed;
        if reset_clock {
            gesture = None;
        }
        while let Some(event) = input.try_next() {
            let mut action = None;
            let mut press = None;
            match event {
                Event::KeyboardInput { event, .. } if event.state.is_pressed() => {
                    if controls.key(&event) {
                        continue;
                    }
                    if event.repeat {
                        continue;
                    }
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Space) => action = Some(Action::Pause),
                        PhysicalKey::Code(KeyCode::KeyP) => action = Some(Action::Panel),
                        PhysicalKey::Code(KeyCode::KeyN) => action = Some(Action::Tool),
                        PhysicalKey::Code(KeyCode::Home) => action = Some(Action::Home),
                        PhysicalKey::Code(KeyCode::Equal | KeyCode::NumpadAdd) => {
                            action = Some(Action::ZoomIn)
                        }
                        PhysicalKey::Code(KeyCode::Minus | KeyCode::NumpadSubtract) => {
                            action = Some(Action::ZoomOut)
                        }
                        PhysicalKey::Code(KeyCode::Escape) => {
                            if gesture.take().is_some() {
                                controls.message.clear();
                            } else {
                                frame.discard();
                                break 'frames;
                            }
                        }
                        _ => {}
                    }
                }
                Event::CursorMoved { position, .. } => {
                    let point = Vec2::new(position.x as f32, position.y as f32);
                    cursor = Some(point);
                    let point = point / scale_factor as f32;
                    if matches!(gesture, Some((Pointer::Mouse(_), _))) {
                        move_pointer(point, &mut gesture, &mut controls, &mut view, world_size);
                    }
                }
                Event::CursorLeft { .. } => {
                    cursor = None;
                    gesture = None;
                    controls.message.clear();
                }
                Event::MouseInput { button, state, .. } => {
                    if let Some(point) = cursor.map(|p| p / scale_factor as f32) {
                        if state.is_pressed() {
                            press = Some((Pointer::Mouse(button), point));
                        } else {
                            release_pointer(
                                Pointer::Mouse(button),
                                point,
                                &mut gesture,
                                &mut controls,
                                &mut simulation,
                                size,
                            );
                        }
                    }
                }
                Event::MouseWheel { delta, .. } if gesture.is_none() => {
                    if let Some(point) = cursor.map(|p| p / scale_factor as f32)
                        && !controls.covers(point, size)
                    {
                        let amount = match delta {
                            MouseScrollDelta::LineDelta(_, y) => f64::from(y) * 0.16,
                            MouseScrollDelta::PixelDelta(p) => p.y / scale_factor * 0.002,
                        };
                        view.zoom_at(amount.clamp(-2.0, 2.0).exp(), point, world_size);
                    }
                }
                Event::Touch(touch) => {
                    let point = Vec2::new(touch.location.x as f32, touch.location.y as f32)
                        / scale_factor as f32;
                    let pointer = Pointer::Touch(touch.id);
                    match touch.phase {
                        TouchPhase::Started => press = Some((pointer, point)),
                        TouchPhase::Moved
                            if gesture.as_ref().is_some_and(|(id, _)| *id == pointer) =>
                        {
                            move_pointer(point, &mut gesture, &mut controls, &mut view, world_size)
                        }
                        TouchPhase::Ended => {
                            if gesture.as_ref().is_some_and(|(id, _)| *id == pointer) {
                                move_pointer(
                                    point,
                                    &mut gesture,
                                    &mut controls,
                                    &mut view,
                                    world_size,
                                );
                                release_pointer(
                                    pointer,
                                    point,
                                    &mut gesture,
                                    &mut controls,
                                    &mut simulation,
                                    size,
                                );
                            }
                        }
                        TouchPhase::Cancelled
                            if gesture.as_ref().is_some_and(|(id, _)| *id == pointer) =>
                        {
                            gesture = None
                        }
                        _ => {}
                    }
                }
                Event::Focused(value) => {
                    focused = value;
                    cursor = None;
                    gesture = None;
                    controls.message.clear();
                    controls.blur();
                    reset_clock = true;
                }
                _ => {}
            }
            if let Some((pointer, point)) = press
                && gesture.is_none()
            {
                if controls.covers(point, size) {
                    if matches!(
                        pointer,
                        Pointer::Mouse(MouseButton::Left) | Pointer::Touch(_)
                    ) {
                        action = controls.hit_test(point, size);
                    }
                } else {
                    controls.blur();
                    let pan = matches!(
                        pointer,
                        Pointer::Mouse(MouseButton::Right | MouseButton::Middle)
                    );
                    let primary = matches!(
                        pointer,
                        Pointer::Mouse(MouseButton::Left) | Pointer::Touch(_)
                    );
                    if pan || (primary && !controls.launch) {
                        gesture = Some((pointer, Gesture::Pan { last: point }));
                    } else if primary {
                        match controls.spec() {
                            Ok(spec) => {
                                controls.message = "Release to launch; Esc to cancel".into();
                                gesture = Some((
                                    pointer,
                                    Gesture::Launch {
                                        pixel: point,
                                        position: view.world(point, world_size),
                                        spec,
                                        aimed: false,
                                    },
                                ));
                            }
                            Err(error) => controls.message = error.into(),
                        }
                    }
                }
            }
            if let Some(action) = action {
                gesture = None;
                controls.apply(action);
                world_size = controls.world_size(size);
                match action {
                    Action::Home => view.home(world_size),
                    Action::Reset => {
                        simulation = Simulation::solar_system();
                        view.home(world_size);
                        reset_clock = true;
                        controls.message.clear();
                    }
                    Action::ZoomIn => view.zoom_at(1.25, world_size * 0.5, world_size),
                    Action::ZoomOut => view.zoom_at(0.8, world_size * 0.5, world_size),
                    Action::Pause => reset_clock = true,
                    _ => {}
                }
            }
        }
        let now = Instant::now();
        let elapsed = now - last;
        last = now;
        let aiming = gesture.as_ref().is_some_and(|(_, g)| g.is_launch());
        let running = !controls.paused && (focused || smoke) && !aiming;
        if reset_clock {
            clock.reset();
        }
        for _ in 0..clock.advance(elapsed, running && !reset_clock) {
            simulation.step();
        }

        frame.clear(color::BLACK);
        let camera = frame.logical_camera().transform(view.transform(world_size));
        let mut scene = frame.scene();
        scene.camera = camera;
        simulation.draw(&library, &mut scene);
        if let Some((_, gesture)) = &gesture {
            gesture.draw(&library, &mut scene, view.zoom);
        }
        scene.render();
        let camera = frame.logical_camera();
        let mut overlay = frame.scene();
        overlay.camera = camera;
        controls.draw(
            &library,
            &mut overlay,
            size,
            simulation.body_count(),
            view.zoom,
            scale_factor,
        );
        overlay.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            println!(
                "Smoke check complete: {frames} frames, {} bodies, {:.3} simulation seconds",
                simulation.body_count(),
                simulation.time()
            );
            break;
        }
    }
    Ok(())
}
