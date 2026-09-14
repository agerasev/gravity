#![forbid(unsafe_code)]

mod app;
mod controls;
mod timing;

#[wgame::window(title = "Gravity", size = (1200, 900), resizable = true, vsync = true)]
async fn main(window: wgame::Window<'_>) -> wgame::Result<()> {
    app::run(window).await
}
