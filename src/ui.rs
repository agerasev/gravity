use crate::controls::{Action, Controls};
use wgame_egui::{Canvas, egui};

pub fn layout(ui: &mut egui::Ui, canvas: &Canvas, controls: &mut Controls) -> egui::Response {
    egui::Panel::top("status").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("GRAVITY");
            ui.label(format!(
                "{} bodies | {:.0}% zoom",
                controls.bodies,
                controls.zoom * 100.0
            ));
            action_button(
                ui,
                controls,
                Action::Panel,
                if controls.panel {
                    "Hide controls [P]"
                } else {
                    "Show controls [P]"
                },
            );
            ui.label(if controls.paused { "Paused" } else { "Running" });
        });
    });
    if controls.panel {
        egui::Panel::right("controls")
            .default_size(300.0)
            .size_range(180.0..=400.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| controls_panel(ui, controls));
            });
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ui, |ui| canvas.show(ui))
        .inner
}

fn action_button(ui: &mut egui::Ui, controls: &mut Controls, action: Action, label: &str) {
    if ui.button(label).clicked() {
        controls.actions.push(action);
    }
}

fn text_value(ui: &mut egui::Ui, controls: &mut Controls, field: usize, width: f32) {
    if ui
        .add(
            egui::TextEdit::singleline(&mut controls.values[field])
                .char_limit(24)
                .desired_width(width),
        )
        .changed()
    {
        controls.message.clear();
    }
}

fn controls_panel(ui: &mut egui::Ui, controls: &mut Controls) {
    ui.heading("Orbital playground");
    ui.horizontal_wrapped(|ui| {
        action_button(
            ui,
            controls,
            Action::Pause,
            if controls.paused {
                "Resume [Space]"
            } else {
                "Pause [Space]"
            },
        );
        action_button(ui, controls, Action::Reset, "Reset system");
    });
    ui.horizontal_wrapped(|ui| {
        action_button(ui, controls, Action::Home, "Home view");
        action_button(ui, controls, Action::ZoomIn, "Zoom +");
        action_button(ui, controls, Action::ZoomOut, "Zoom −");
    });
    action_button(
        ui,
        controls,
        Action::Tool,
        if controls.launch {
            "Tool: launch body [N]"
        } else {
            "Tool: pan view [N]"
        },
    );
    let mut collisions = controls.collisions;
    if ui
        .checkbox(&mut collisions, "Merge collisions [C]")
        .changed()
    {
        controls.actions.push(Action::Collisions);
    }
    ui.separator();
    ui.strong("New body");
    ui.label("Mass (0.01–100000)");
    text_value(ui, controls, 0, 100.0);
    let mut mass = controls.values[0]
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .unwrap_or(1.0)
        .clamp(0.01, 100_000.0);
    let slider = ui.scope(|ui| {
        ui.spacing_mut().slider_width = ui.available_width();
        ui.add(
            egui::Slider::new(&mut mass, 0.01..=100_000.0)
                .logarithmic(true)
                .show_value(false)
                .max_decimals(3),
        )
    });
    if slider
        .inner
        .on_hover_text("Drag to adjust mass across small moons, planets, and stars.")
        .changed()
    {
        controls.values[0] = mass.to_string();
        controls.message.clear();
    }
    ui.label("Color");
    ui.horizontal(|ui| {
        let mut rgb = controls.color_rgb().unwrap_or([112, 207, 255]);
        if ui
            .color_edit_button_srgb(&mut rgb)
            .on_hover_text("Choose body color")
            .changed()
        {
            controls.set_color_rgb(rgb);
            controls.message.clear();
        }
        text_value(ui, controls, 1, ui.available_width());
    });
    ui.separator();
    ui.label("Click to place a stationary body, or drag from the spawn point to set its velocity.");
    ui.label(format!(
        "Forecast: {}× speed. Move at least {} pixels to change the aim.",
        crate::gesture::FORECAST_SPEED,
        crate::gesture::AIM_THRESHOLD
    ));
    ui.label("Right/middle drag: pan. Wheel: zoom at cursor. Home: fit.");
    ui.label("Click the playground for shortcuts. Esc: cancel a launch, then close.");
    if let Err(error) = controls.spec() {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
    } else {
        ui.label(&controls.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_widgets_does_not_overwrite_partial_text_input() {
        let context = egui::Context::default();
        let mut controls = Controls {
            values: ["1e".into(), "#12".into()],
            ..Default::default()
        };
        for _ in 0..3 {
            let mut output = context.run_ui(egui::RawInput::default(), |ui| {
                controls_panel(ui, &mut controls);
            });
            output.textures_delta.clear();
        }
        assert_eq!(controls.values, ["1e", "#12"]);
        assert!(controls.spec().is_err());
    }
}
