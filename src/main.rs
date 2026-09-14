#![forbid(unsafe_code)]

mod app;
mod controls;
mod gesture;
mod timing;
mod ui;
mod viewport;

#[wgame::window(title = "Gravity", logical_size = (1200.0, 900.0), resizable = true, vsync = true)]
async fn main(window: wgame::Window<'_>) -> wgame::Result<()> {
    let controls = std::rc::Rc::new(std::cell::RefCell::new(controls::Controls::default()));
    let ui_controls = controls.clone();
    let host = wgame_egui::EguiWindow::new(window, move |ui, canvas| {
        ui::layout(ui, canvas, &mut ui_controls.borrow_mut())
    });
    app::run(host, controls).await
}
