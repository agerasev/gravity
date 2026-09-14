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
            if ui
                .button(if controls.panel {
                    "Hide controls [P]"
                } else {
                    "Show controls [P]"
                })
                .clicked()
            {
                controls.actions.push(Action::Panel);
            }
            ui.label(if controls.paused { "Paused" } else { "Running" });
        });
    });
    if controls.panel {
        egui::Panel::right("controls").default_size(300.0).size_range(180.0..=400.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Orbital playground");
                for (action, label) in [
                    (Action::Pause, if controls.paused { "Resume [Space]" } else { "Pause [Space]" }),
                    (Action::Home, "Home view"), (Action::Reset, "Reset system"),
                    (Action::ZoomIn, "Zoom +"), (Action::ZoomOut, "Zoom -"),
                    (Action::Tool, if controls.launch { "Tool: launch body [N]" } else { "Tool: pan view [N]" }),
                ] {
                    if ui.button(label).clicked() { controls.actions.push(action); }
                }
                let mut collisions = controls.collisions;
                if ui.checkbox(&mut collisions, "Merge collisions [C]").changed() {
                    controls.actions.push(Action::Collisions);
                }
                ui.separator();
                for (value, label) in controls.values.iter_mut().zip([
                    "Mass (0.01–100000)", "Color (#RRGGBB)", "X velocity / sec (right +)", "Y velocity / sec (down +)",
                ]) { ui.label(label); ui.add(egui::TextEdit::singleline(value).char_limit(24)); }
                ui.separator();
                ui.label("Click to launch with the entered velocity, or drag from the spawn point to aim.");
                ui.label("Right/middle drag: pan. Wheel: zoom at cursor. Home: fit.");
                ui.label("Click the playground for shortcuts. Esc: cancel a launch, then close.");
                if let Err(error) = controls.spec() { ui.colored_label(egui::Color32::LIGHT_RED, error); }
                else { ui.label(&controls.message); }
            });
        });
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ui, |ui| canvas.show(ui))
        .inner
}
