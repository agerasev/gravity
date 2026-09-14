use gravity::BodySpec;
use wgame::{
    Library, Result,
    gfx::Scene,
    glam::{DVec2, Vec2},
    input::{
        event::KeyEvent,
        keyboard::{Key, NamedKey},
    },
    prelude::*,
    rgb::Rgba,
    typography::{FontData, FontTexture, Text},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Panel,
    Pause,
    Home,
    Reset,
    ZoomIn,
    ZoomOut,
    Tool,
    Field(usize),
}

#[derive(Clone, Copy)]
struct Rect {
    min: Vec2,
    max: Vec2,
}
impl Rect {
    fn contains(self, p: Vec2) -> bool {
        p.cmpge(self.min).all() && p.cmplt(self.max).all()
    }
}

struct Layout {
    origin: Vec2,
    scale: f32,
}
impl Layout {
    fn new(size: Vec2) -> Self {
        let scale = (size.y / 740.0).min(size.x / 340.0).clamp(0.01, 1.0);
        Self {
            origin: Vec2::new(size.x - 330.0 * scale, 0.0),
            scale,
        }
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            min: self.origin + Vec2::new(x, y) * self.scale,
            max: self.origin + Vec2::new(x + w, y + h) * self.scale,
        }
    }
    fn button(&self, action: Action) -> Rect {
        match action {
            Action::Panel => self.rect(12.0, 12.0, 306.0, 36.0),
            Action::Pause => self.rect(12.0, 58.0, 306.0, 36.0),
            Action::Home => self.rect(12.0, 104.0, 147.0, 36.0),
            Action::Reset => self.rect(171.0, 104.0, 147.0, 36.0),
            Action::ZoomIn => self.rect(12.0, 150.0, 147.0, 36.0),
            Action::ZoomOut => self.rect(171.0, 150.0, 147.0, 36.0),
            Action::Tool => self.rect(12.0, 236.0, 306.0, 36.0),
            Action::Field(i) => self.rect(12.0, 314.0 + i as f32 * 72.0, 306.0, 36.0),
        }
    }
}

struct Editor {
    values: [String; 4],
    active: Option<usize>,
    replace: bool,
    original: String,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            values: ["1", "#70CFFF", "0", "-50"].map(String::from),
            active: None,
            replace: false,
            original: String::new(),
        }
    }
}
impl Editor {
    fn focus(&mut self, field: usize) {
        self.active = Some(field);
        self.original = self.values[field].clone();
        self.replace = true;
    }
    fn key(&mut self, key: &Key, text: Option<&str>) -> bool {
        let Some(i) = self.active else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Escape) => {
                self.values[i] = self.original.clone();
                self.active = None;
            }
            Key::Named(NamedKey::Enter) => self.active = None,
            Key::Named(NamedKey::Tab) => self.focus((i + 1) % 4),
            Key::Named(NamedKey::Backspace | NamedKey::Delete) => {
                if self.replace {
                    self.values[i].clear();
                } else {
                    self.values[i].pop();
                }
                self.replace = false;
            }
            _ => {
                if let Some(text) = text {
                    let filtered: String = text
                        .chars()
                        .filter(|c| {
                            if i == 1 {
                                c.is_ascii_hexdigit() || *c == '#'
                            } else {
                                c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')
                            }
                        })
                        .collect();
                    if !filtered.is_empty() {
                        if self.replace {
                            self.values[i].clear();
                            self.replace = false;
                        }
                        if self.values[i].len() + filtered.len() <= 24 {
                            self.values[i].push_str(&filtered);
                        }
                    }
                }
            }
        }
        true
    }
    fn spec(&self) -> std::result::Result<BodySpec, &'static str> {
        let mass = self.values[0].parse().map_err(|_| "Enter a valid mass")?;
        let hex = self.values[1].strip_prefix('#').unwrap_or(&self.values[1]);
        if hex.len() != 6 {
            return Err("Color needs 6 hex digits");
        }
        let rgb = u32::from_str_radix(hex, 16).map_err(|_| "Color needs 6 hex digits")?;
        let spec = BodySpec {
            mass,
            color: Rgba::new(
                ((rgb >> 16) & 255) as f32 / 255.0,
                ((rgb >> 8) & 255) as f32 / 255.0,
                (rgb & 255) as f32 / 255.0,
                1.0,
            ),
            velocity: DVec2::new(
                self.values[2]
                    .parse()
                    .map_err(|_| "Enter a valid X velocity")?,
                self.values[3]
                    .parse()
                    .map_err(|_| "Enter a valid Y velocity")?,
            ),
        };
        spec.validate()?;
        Ok(spec)
    }
}

pub struct Controls {
    pub panel: bool,
    pub paused: bool,
    pub launch: bool,
    pub message: String,
    editor: Editor,
    font: FontTexture,
    labels: Vec<(String, Text)>,
}
impl Controls {
    pub fn new(library: &Library) -> Result<Self> {
        let font = library.make_font(&FontData::new(
            include_bytes!("../assets/DejaVuSans.ttf").to_vec(),
            0,
        )?);
        Ok(Self {
            panel: true,
            paused: false,
            launch: true,
            message: String::new(),
            editor: Editor::default(),
            font: font.rasterize(20.0),
            labels: Vec::new(),
        })
    }
    pub fn world_size(&self, size: Vec2) -> Vec2 {
        if self.panel {
            Vec2::new(Layout::new(size).origin.x, size.y)
        } else {
            size
        }
    }
    pub fn covers(&self, p: Vec2, size: Vec2) -> bool {
        let l = Layout::new(size);
        if self.panel {
            p.x >= l.origin.x
        } else {
            l.button(Action::Panel).contains(p)
        }
    }
    pub fn hit_test(&self, p: Vec2, size: Vec2) -> Option<Action> {
        let l = Layout::new(size);
        [
            Action::Panel,
            Action::Pause,
            Action::Home,
            Action::Reset,
            Action::ZoomIn,
            Action::ZoomOut,
            Action::Tool,
            Action::Field(0),
            Action::Field(1),
            Action::Field(2),
            Action::Field(3),
        ]
        .into_iter()
        .filter(|a| self.panel || *a == Action::Panel)
        .find(|a| l.button(*a).contains(p))
    }
    pub fn apply(&mut self, action: Action) {
        self.editor.active = None;
        match action {
            Action::Panel => self.panel = !self.panel,
            Action::Pause => self.paused = !self.paused,
            Action::Tool => {
                self.launch = !self.launch;
                self.message.clear();
            }
            Action::Field(i) => {
                self.message.clear();
                self.editor.focus(i);
            }
            _ => {}
        }
    }
    pub fn key(&mut self, event: &KeyEvent) -> bool {
        self.editor.key(&event.logical_key, event.text.as_deref())
    }
    pub fn blur(&mut self) {
        self.editor.active = None;
    }
    pub fn spec(&self) -> std::result::Result<BodySpec, &'static str> {
        self.editor.spec()
    }
    pub fn set_velocity(&mut self, velocity: DVec2) {
        self.editor.values[2] = format!("{:.2}", velocity.x);
        self.editor.values[3] = format!("{:.2}", velocity.y);
    }
    pub fn draw(
        &mut self,
        library: &Library,
        scene: &mut Scene,
        size: Vec2,
        count: usize,
        zoom: f64,
    ) {
        let l = Layout::new(size);
        if self.panel {
            scene.add(
                &library
                    .shapes()
                    .rectangle((l.origin, size))
                    .fill_color(Rgba::new(0.035, 0.05, 0.08, 0.97)),
            );
        }
        let mut rows: Vec<(String, Vec2, f32)> = Vec::new();
        let mut button = |action, label: String| {
            let rect = l.button(action);
            let active = matches!(action, Action::Field(i) if self.editor.active == Some(i))
                || (action == Action::Tool && self.launch);
            let color = if active {
                Rgba::new(0.13, 0.31, 0.43, 1.0)
            } else {
                Rgba::new(0.10, 0.14, 0.21, 1.0)
            };
            scene.add(
                &library
                    .shapes()
                    .rectangle((rect.min, rect.max))
                    .fill_color(color),
            );
            rows.push((
                label,
                rect.min + Vec2::new(10.0, 24.0) * l.scale,
                16.0 * l.scale,
            ));
        };
        button(
            Action::Panel,
            if self.panel {
                "Hide controls [P]"
            } else {
                "Controls [P]"
            }
            .into(),
        );
        if self.panel {
            button(
                Action::Pause,
                if self.paused {
                    "Resume [Space]"
                } else {
                    "Pause [Space]"
                }
                .into(),
            );
            button(Action::Home, "Home view".into());
            button(Action::Reset, "Reset system".into());
            button(Action::ZoomIn, "Zoom +".into());
            button(Action::ZoomOut, "Zoom -".into());
            button(
                Action::Tool,
                if self.launch {
                    "Tool: launch body [N]"
                } else {
                    "Tool: pan view [N]"
                }
                .into(),
            );
            for i in 0..4 {
                let suffix = if self.editor.active == Some(i) {
                    " |"
                } else {
                    ""
                };
                button(
                    Action::Field(i),
                    format!("{}{suffix}", self.editor.values[i]),
                );
            }
            for (label, y) in [
                ("ORBITAL PLAYGROUND", 219.0),
                ("Mass (0.01 - 100000)", 302.0),
                ("Color (#RRGGBB)", 374.0),
                ("X velocity / sec (right +)", 446.0),
                ("Y velocity / sec (down +)", 518.0),
                ("Launch: click for entered velocity", 589.0),
                ("or drag from spawn point to aim.", 611.0),
                ("Right / middle drag: pan", 640.0),
                ("Wheel: zoom at cursor | Home: fit", 662.0),
                ("Esc: cancel edit / drag, then close", 684.0),
            ] {
                rows.push((
                    label.into(),
                    l.origin + Vec2::new(12.0, y) * l.scale,
                    14.0 * l.scale,
                ));
            }
            if let Ok(spec) = self.spec() {
                scene.add(
                    &library
                        .shapes()
                        .rectangle((
                            l.rect(290.0, 363.0, 24.0, 14.0).min,
                            l.rect(290.0, 363.0, 24.0, 14.0).max,
                        ))
                        .fill_color(spec.color),
                );
            }
            let status = if let Err(error) = self.spec() {
                error.into()
            } else if self.editor.active.is_some() {
                "Type to replace | Enter saves | Tab next".into()
            } else if !self.message.is_empty() {
                self.message.clone()
            } else {
                format!("{count} bodies | {:.0}% zoom", zoom * 100.0)
            };
            rows.push((
                status,
                l.origin + Vec2::new(12.0, 720.0) * l.scale,
                13.0 * l.scale,
            ));
        }
        let status = format!(
            "{} | {}",
            if self.paused { "PAUSED" } else { "GRAVITY" },
            if self.launch { "LAUNCH" } else { "PAN" }
        );
        rows.push((status, Vec2::new(16.0, 28.0), 16.0));
        // One cache slot per visible label, including changing numeric values.
        for (i, (label, pos, scale)) in rows.into_iter().enumerate() {
            if i == self.labels.len() {
                self.labels.push((label.clone(), self.font.text(&label)));
            } else if self.labels[i].0 != label {
                self.labels[i] = (label.clone(), self.font.text(&label));
            }
            scene.add(&self.labels[i].1.scale(scale).move_to(pos));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_replaces_edits_cancels_and_validates() {
        let mut e = Editor::default();
        e.focus(0);
        assert!(e.key(&Key::Character("2".into()), Some("2")));
        assert_eq!(e.spec().unwrap().mass, 2.0);
        e.key(&Key::Named(NamedKey::Escape), None);
        assert_eq!(e.spec().unwrap().mass, 1.0);
        e.values[0] = "NaN".into();
        assert!(e.spec().is_err());
        e.values[0] = "-1".into();
        assert!(e.spec().is_err());
        e.values[0] = "1".into();
        e.values[1] = "abc".into();
        assert!(e.spec().is_err());
        e.values[1] = "#FF0080".into();
        assert_eq!(
            e.spec().unwrap().color,
            Rgba::new(1.0, 0.0, 128.0 / 255.0, 1.0)
        );
        e.values[2] = "inf".into();
        assert!(e.spec().is_err());
    }
    #[test]
    fn scaled_layout_keeps_fields_inside_panel() {
        for size in [Vec2::new(1200.0, 900.0), Vec2::new(320.0, 480.0)] {
            let l = Layout::new(size);
            for action in [Action::Panel, Action::Field(3), Action::Reset] {
                let r = l.button(action);
                assert!(r.min.x >= 0.0 && r.max.x <= size.x && r.max.y <= size.y);
                assert!(r.contains((r.min + r.max) * 0.5));
            }
        }
    }
}
