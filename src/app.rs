use crate::{
    controls::{Action, Controls},
    gesture::Gesture,
    timing::Clock,
    viewport::Viewport,
};
use gravity::Simulation;
use std::{cell::RefCell, rc::Rc};
use wgame::{
    Library, Result, WindowHost,
    app::time::Instant,
    canvas::{Button, Event, Key},
    gfx::types::color,
    glam::Vec2,
    prelude::*,
};

fn move_pointer(
    point: Vec2,
    gesture: &mut Option<(Button, Gesture)>,
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
    pointer: Button,
    point: Vec2,
    gesture: &mut Option<(Button, Gesture)>,
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
        if point.x < 0.0 || point.y < 0.0 || point.x >= size.x || point.y >= size.y {
            controls.message = "Launch cancelled outside playground".into();
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

pub async fn run(mut host: impl WindowHost, shared: Rc<RefCell<Controls>>) -> Result<()> {
    let library = Library::new(host.graphics());
    let mut simulation = Simulation::solar_system();
    let mut view = Viewport::default();
    let mut gesture: Option<(Button, Gesture)> = None;
    let mut clock = Clock::default();
    let mut last = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;
    let mut previous_size = Vec2::ZERO;
    'frames: while let Some(mut frame) = host.next_frame().await? {
        let mut controls = shared.borrow_mut();
        let logical_size = frame.logical_size();
        let size = Vec2::new(logical_size.0 as f32, logical_size.1 as f32);
        if frames == 0 {
            view.home(size);
        }
        let mut reset_clock = size != previous_size;
        previous_size = size;
        if reset_clock {
            gesture = None;
        }
        for event in &frame.input().events {
            match *event {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                } => {
                    let action = match key {
                        Key::Space => Some(Action::Pause),
                        Key::Character('p') => Some(Action::Panel),
                        Key::Character('n') => Some(Action::Tool),
                        Key::Home => Some(Action::Home),
                        Key::Plus => Some(Action::ZoomIn),
                        Key::Minus => Some(Action::ZoomOut),
                        Key::Escape => {
                            if gesture.take().is_some() {
                                controls.message.clear();
                            } else {
                                drop(controls);
                                frame.discard();
                                break 'frames;
                            }
                            None
                        }
                        _ => None,
                    };
                    if let Some(action) = action {
                        controls.actions.push(action);
                    }
                }
                Event::Moved(point) => {
                    move_pointer(point, &mut gesture, &mut controls, &mut view, size)
                }
                Event::Button {
                    button,
                    pressed,
                    position: point,
                } => {
                    if !pressed {
                        move_pointer(point, &mut gesture, &mut controls, &mut view, size);
                        release_pointer(
                            button,
                            point,
                            &mut gesture,
                            &mut controls,
                            &mut simulation,
                            size,
                        );
                    } else if gesture.is_none() {
                        let pan = matches!(button, Button::Secondary | Button::Middle);
                        let primary = button == Button::Primary;
                        if pan || (primary && !controls.launch) {
                            gesture = Some((button, Gesture::Pan { last: point }));
                        } else if primary {
                            match controls.spec() {
                                Ok(spec) => {
                                    controls.message = "Release to launch; Esc to cancel".into();
                                    gesture = Some((
                                        button,
                                        Gesture::Launch {
                                            pixel: point,
                                            position: view.world(point, size),
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
                Event::Scroll(delta) if gesture.is_none() => {
                    if let Some(point) = frame.input().pointer {
                        view.zoom_at(
                            (f64::from(delta.y) * 0.004).clamp(-2.0, 2.0).exp(),
                            point,
                            size,
                        );
                    }
                }
                Event::Cancelled | Event::Focused(_) => {
                    gesture = None;
                    controls.message.clear();
                    reset_clock = true;
                }
                _ => {}
            }
        }
        for action in std::mem::take(&mut controls.actions) {
            gesture = None;
            controls.apply(action);
            match action {
                Action::Home => view.home(size),
                Action::Reset => {
                    simulation = Simulation::solar_system();
                    view.home(size);
                    reset_clock = true;
                    controls.message.clear();
                }
                Action::ZoomIn => view.zoom_at(1.25, size * 0.5, size),
                Action::ZoomOut => view.zoom_at(0.8, size * 0.5, size),
                Action::Pause => reset_clock = true,
                _ => {}
            }
        }
        let now = Instant::now();
        let elapsed = now - last;
        last = now;
        let aiming = gesture.as_ref().is_some_and(|(_, g)| g.is_launch());
        let running = !controls.paused
            && (frame.input().window_focused || smoke)
            && !aiming
            && frame.visible();
        if reset_clock {
            clock.reset();
        }
        for _ in 0..clock.advance(elapsed, running && !reset_clock) {
            simulation.step();
        }
        controls.bodies = simulation.body_count();
        controls.zoom = view.zoom;
        drop(controls);
        frame.clear(color::BLACK);
        let camera = frame.logical_camera().transform(view.transform(size));
        let mut scene = frame.scene();
        scene.camera = camera;
        simulation.draw(&library, &mut scene);
        if let Some((_, gesture)) = &gesture {
            gesture.draw(&library, &mut scene, view.zoom);
        }
        scene.render();
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
