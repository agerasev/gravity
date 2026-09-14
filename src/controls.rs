use gravity::BodySpec;
use wgame::{glam::DVec2, rgb::Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Panel,
    Pause,
    Home,
    Reset,
    ZoomIn,
    ZoomOut,
    Tool,
}

/// Application settings and actions; no GUI types enter the simulation loop.
pub struct Controls {
    pub panel: bool,
    pub paused: bool,
    pub launch: bool,
    pub message: String,
    pub values: [String; 4],
    pub actions: Vec<Action>,
    pub bodies: usize,
    pub zoom: f64,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            panel: true,
            paused: false,
            launch: true,
            message: String::new(),
            values: ["1", "#70CFFF", "0", "-50"].map(String::from),
            actions: Vec::new(),
            bodies: 0,
            zoom: 1.0,
        }
    }
}
impl Controls {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Panel => self.panel = !self.panel,
            Action::Pause => self.paused = !self.paused,
            Action::Tool => {
                self.launch = !self.launch;
                self.message.clear();
            }
            _ => {}
        }
    }
    pub fn set_velocity(&mut self, velocity: DVec2) {
        self.values[2] = format!("{:.2}", velocity.x);
        self.values[3] = format!("{:.2}", velocity.y);
    }
    pub fn spec(&self) -> std::result::Result<BodySpec, &'static str> {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edited_body_settings_are_validated_before_launch() {
        let mut controls = Controls::default();
        assert_eq!(controls.spec().unwrap().mass, 1.0);
        for value in ["NaN", "-1", "invalid"] {
            controls.values[0] = value.into();
            assert!(controls.spec().is_err());
        }
        controls.values[0] = "2".into();
        controls.values[1] = "#FF0080".into();
        assert_eq!(
            controls.spec().unwrap().color,
            Rgba::new(1.0, 0.0, 128.0 / 255.0, 1.0)
        );
        controls.values[2] = "inf".into();
        assert!(controls.spec().is_err());
    }
}
