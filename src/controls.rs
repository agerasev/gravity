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
    Collisions,
}

/// Application settings and actions; no GUI types enter the simulation loop.
pub struct Controls {
    pub panel: bool,
    pub paused: bool,
    pub launch: bool,
    pub collisions: bool,
    pub message: String,
    pub values: [String; 2],
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
            collisions: true,
            message: String::new(),
            values: ["1", "#70CFFF"].map(String::from),
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
            Action::Collisions => self.collisions = !self.collisions,
            Action::Tool => {
                self.launch = !self.launch;
                self.message.clear();
            }
            _ => {}
        }
    }
    pub fn color_rgb(&self) -> std::result::Result<[u8; 3], &'static str> {
        let hex = self.values[1].strip_prefix('#').unwrap_or(&self.values[1]);
        if hex.len() != 6 {
            return Err("Color needs 6 hex digits");
        }
        let rgb = u32::from_str_radix(hex, 16).map_err(|_| "Color needs 6 hex digits")?;
        Ok([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8])
    }

    pub fn set_color_rgb(&mut self, rgb: [u8; 3]) {
        self.values[1] = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
    }

    pub fn spec(&self) -> std::result::Result<BodySpec, &'static str> {
        let mass = self.values[0].parse().map_err(|_| "Enter a valid mass")?;
        let [r, g, b] = self.color_rgb()?;
        let spec = BodySpec {
            mass,
            color: Rgba::new(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                1.0,
            ),
            velocity: DVec2::ZERO,
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
        assert_eq!(controls.spec().unwrap().velocity, DVec2::ZERO);
    }
    #[test]
    fn color_picker_and_hex_input_share_the_launch_color() {
        let mut controls = Controls::default();
        for rgb in [[0, 0, 0], [255, 255, 255], [35, 128, 241]] {
            controls.set_color_rgb(rgb);
            assert_eq!(controls.color_rgb().unwrap(), rgb);
            let color = controls.spec().unwrap().color;
            assert_eq!(
                [color.r, color.g, color.b],
                rgb.map(|v| f32::from(v) / 255.0)
            );
        }
        controls.values[1] = "aB10fF".into();
        assert_eq!(controls.color_rgb().unwrap(), [171, 16, 255]);
        controls.values[1] = "#xyz123".into();
        assert!(controls.spec().is_err());
    }
}
