use gravity::Simulation;
use wgame::{
    Event, Library, Result, Window,
    app::time::Instant,
    gfx::types::color,
    glam::{Affine2, Vec2},
    input::{
        event::{MouseButton, TouchPhase},
        keyboard::{KeyCode, PhysicalKey},
    },
    prelude::*,
};

use crate::{
    controls::{Action, Controls},
    timing::Clock,
};

pub async fn run(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let mut controls = Controls::new(&library)?;
    let mut simulation = Simulation::new(getrandom::u64()?);
    let mut input = window.input();
    let mut cursor = None;
    let mut focused = true;
    let mut clock = Clock::default();
    let mut last = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;

    'frames: while let Some(mut frame) = window.next_frame().await? {
        let size = Vec2::new(frame.size().0 as f32, frame.size().1 as f32);
        let mut reset_clock = frame.resized().is_some();
        while let Some(event) = input.try_next() {
            let action = match event {
                Event::KeyboardInput { event, .. } if event.state.is_pressed() && !event.repeat => {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Space) => Some(Action::Pause),
                        PhysicalKey::Code(KeyCode::KeyP) => Some(Action::Panel),
                        PhysicalKey::Code(KeyCode::Escape) => {
                            frame.discard();
                            break 'frames;
                        }
                        _ => None,
                    }
                }
                Event::CursorMoved { position, .. } => {
                    cursor = Some(Vec2::new(position.x as f32, position.y as f32));
                    None
                }
                Event::CursorLeft { .. } => {
                    cursor = None;
                    None
                }
                Event::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } if state.is_pressed() => cursor.and_then(|point| controls.hit_test(point, size)),
                Event::Touch(touch) if touch.phase == TouchPhase::Started => controls.hit_test(
                    Vec2::new(touch.location.x as f32, touch.location.y as f32),
                    size,
                ),
                Event::Focused(value) => {
                    focused = value;
                    cursor = None;
                    reset_clock = true;
                    None
                }
                _ => None,
            };
            if let Some(action) = action {
                reset_clock |= controls.apply(action);
            }
        }
        let now = Instant::now();
        let elapsed = now - last;
        last = now;
        // Smoke checks must exercise growing trails even when launched behind
        // another window; ordinary runs retain focus-based pausing.
        let running = !controls.paused() && (focused || smoke);
        if reset_clock {
            clock.reset();
        }
        for _ in 0..clock.advance(elapsed, running && !reset_clock) {
            simulation.step();
        }

        frame.clear(color::BLACK);
        let camera = frame
            .physical_camera()
            .transform(Affine2::from_translation(size * 0.5));
        let mut scene = frame.scene();
        scene.camera = camera;
        simulation.draw(&library, &mut scene);
        scene.render();

        let camera = frame.physical_camera();
        let mut overlay = frame.scene();
        overlay.camera = camera;
        controls.draw(&library, &mut overlay, size);
        overlay.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            println!(
                "Smoke check complete: {frames} frames, {:.3} simulation seconds",
                simulation.time()
            );
            break;
        }
    }
    Ok(())
}
