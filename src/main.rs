#![forbid(unsafe_code)]

mod app;
mod controls;
mod gesture;
mod timing;
mod viewport;

#[wgame::window(title = "Gravity", logical_size = (1200.0, 900.0), resizable = true, vsync = true)]
async fn main(window: wgame::Window<'_>) -> wgame::Result<()> {
    app::run(window).await
}
