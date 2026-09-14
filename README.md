# Gravity

A 64-body gravitational simulation using `wgame` for desktop/WebGL2 rendering
and `phy` for RK4 integration. Checkouts of both libraries must be adjacent to
this project (`../wgame` and `../phy`). The `wgame` checkout needs the four-point
quad and variable-width polyline APIs.

```sh
cargo run --locked --release
```

Click **Controls** to open the pause panel. Space pauses/resumes, P shows/hides
the panel, and Escape closes the application. Touch can activate the panel
buttons. Losing focus pauses the simulation automatically; it resumes on focus
return unless manually paused. Resizing recenters the view without moving the
bodies or clearing their trails.

Each launch generates a fresh random starting state. Physics uses 240 fixed RK4
steps per second, with at most 100 ms of catch-up after a stall. State and force
calculations use `f64`; `phy` supplies `f32` time steps and solver coefficients.
The original softened force law is retained, with zero acceleration between
exactly coincident bodies instead of division by zero.

Trails sample every 0.2 seconds and retain 6.4 seconds of history. They taper
with age at constant 50% opacity, using straight polyline segments with miter
limit 4 and bevel fallback. The oldest segment is clipped continuously between
samples. Caps are flat, and self-overlapping trails blend more than once.

## Web

Install Trunk and the Rust wasm target, then build static assets:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
./scripts/build-web.sh /gravity/
```

The output is in `dist/`. Omit the deployment path for relative URLs. For local
development, run:

```sh
NO_COLOR=true trunk serve --no-default-features --features web
```

The browser needs WebGL2; click the canvas to focus keyboard controls. Refresh
after Escape to restart. Desktop and web features are mutually exclusive.
The font is embedded; its license is in `assets/DejaVuSans-LICENSE.txt` and is
copied into web builds.

## Verification

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --smoke
```

The smoke mode opens a real window, advances even when unfocused, renders 12
frames, then exits. CPU tests
cover force symmetry, coincident bodies, precision, trail trimming, time
accumulation, and control hit testing.
