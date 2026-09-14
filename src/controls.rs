use wgame::{
    Library, Result,
    gfx::Scene,
    glam::Vec2,
    prelude::*,
    rgb::Rgba,
    typography::{FontData, Text},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Panel,
    Pause,
}

#[derive(Default)]
struct State {
    panel: bool,
    paused: bool,
}

#[derive(Clone, Copy)]
struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn contains(self, point: Vec2) -> bool {
        point.cmpge(self.min).all() && point.cmplt(self.max).all()
    }
}

struct Layout {
    panel: Rect,
    toggle: Rect,
    pause: Rect,
}

impl Layout {
    fn new(size: Vec2, panel: bool) -> Self {
        let width = if panel {
            (0.4 * size.x).clamp(220.0, 360.0)
        } else {
            168.0
        }
        .min(size.x);
        let left = size.x - width;
        let margin = 12.0_f32.min(width * 0.05);
        let min_x = left + margin;
        let max_x = size.x - margin;
        Self {
            panel: Rect {
                min: Vec2::new(left, 0.0),
                max: size,
            },
            toggle: Rect {
                min: Vec2::new(min_x, 12.0),
                max: Vec2::new(max_x, 52.0),
            },
            pause: Rect {
                min: Vec2::new(min_x, 68.0),
                max: Vec2::new(max_x, 108.0),
            },
        }
    }
}

impl State {
    fn hit_test(&self, point: Vec2, size: Vec2) -> Option<Action> {
        let layout = Layout::new(size, self.panel);
        if layout.toggle.contains(point) {
            Some(Action::Panel)
        } else if self.panel && layout.pause.contains(point) {
            Some(Action::Pause)
        } else {
            None
        }
    }

    fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Panel => {
                self.panel = !self.panel;
                false
            }
            Action::Pause => {
                self.paused = !self.paused;
                true
            }
        }
    }
}

pub struct Controls {
    state: State,
    show: Text,
    hide: Text,
    pause: Text,
    resume: Text,
    paused_label: Text,
    help: Vec<Text>,
}

impl Controls {
    pub fn new(library: &Library) -> Result<Self> {
        let font = library.make_font(&FontData::new(
            include_bytes!("../assets/DejaVuSans.ttf").to_vec(),
            0,
        )?);
        let raster = font.rasterize(20.0);
        Ok(Self {
            state: State::default(),
            show: raster.text("Controls"),
            hide: raster.text("Hide controls"),
            pause: raster.text("Pause"),
            resume: raster.text("Resume"),
            paused_label: raster.text("Paused"),
            help: [
                "Space: pause / resume",
                "P: show / hide controls",
                "Esc: close",
                "",
                "Pauses when unfocused.",
            ]
            .map(|line| raster.text(line))
            .into(),
        })
    }

    pub fn paused(&self) -> bool {
        self.state.paused
    }
    pub fn apply(&mut self, action: Action) -> bool {
        self.state.apply(action)
    }
    pub fn hit_test(&self, point: Vec2, size: Vec2) -> Option<Action> {
        self.state.hit_test(point, size)
    }

    pub fn draw(&self, library: &Library, scene: &mut Scene, size: Vec2) {
        let layout = Layout::new(size, self.state.panel);
        let button = |scene: &mut Scene, bounds: Rect, label: &Text| {
            scene.add(
                &library
                    .shapes()
                    .rectangle((bounds.min, bounds.max))
                    .fill_color(Rgba::new(0.12, 0.16, 0.22, 0.96)),
            );
            let scale = ((bounds.max.x - bounds.min.x) / 150.0).min(1.0);
            scene.add(
                &label
                    .scale(20.0 * scale)
                    .move_to(bounds.min + Vec2::new(12.0 * scale, 27.0)),
            );
        };
        if self.state.panel {
            scene.add(
                &library
                    .shapes()
                    .rectangle((layout.panel.min, layout.panel.max))
                    .fill_color(Rgba::new(0.03, 0.04, 0.06, 0.9)),
            );
            button(scene, layout.toggle, &self.hide);
            button(
                scene,
                layout.pause,
                if self.state.paused {
                    &self.resume
                } else {
                    &self.pause
                },
            );
            let scale = ((layout.panel.max.x - layout.panel.min.x) / 220.0).min(1.0);
            for (i, line) in self.help.iter().enumerate() {
                scene.add(
                    &line
                        .scale(16.0 * scale)
                        .move_to(Vec2::new(layout.toggle.min.x, 150.0 + 26.0 * i as f32)),
                );
            }
        } else {
            button(scene, layout.toggle, &self.show);
            if self.state.paused {
                scene.add(
                    &self
                        .paused_label
                        .scale(16.0)
                        .move_to(layout.toggle.min + Vec2::new(12.0, 64.0)),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_follow_resize_and_hidden_panel_cannot_pause() {
        let mut state = State::default();
        for size in [Vec2::new(1200.0, 900.0), Vec2::new(320.0, 480.0)] {
            let toggle = Vec2::new(size.x - 30.0, 30.0);
            let pause = Vec2::new(size.x - 30.0, 85.0);
            assert_eq!(state.hit_test(toggle, size), Some(Action::Panel));
            assert_eq!(state.hit_test(pause, size), None);
            assert!(!state.apply(Action::Panel));
            assert_eq!(state.hit_test(pause, size), Some(Action::Pause));
            assert!(state.apply(Action::Pause));
            assert!(state.paused);
            state.apply(Action::Pause);
            state.apply(Action::Panel);
        }
    }
}
