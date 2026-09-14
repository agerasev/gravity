# Gravity

A gravitational sandbox using `wgame` for desktop/WebGL2 rendering and `phy`
for RK4 integration. Both libraries are pinned Git submodules inside this repo.

Clone with submodules, or initialize them in an existing checkout:

```sh
git clone --recurse-submodules https://github.com/agerasev/gravity.git
cd gravity
# For an existing checkout (also after pulling new submodule revisions):
git submodule update --init --recursive
```

```sh
cargo run --locked --release
```

The default scene is a fictional solar system: a sun, seven planets, and two
moons orbiting the gas giants. Distances, visual sizes, masses, and orbital periods
are chosen for a compact, readable simulation rather than astronomical accuracy.
All bodies attract one another, including the sun; none is pinned in place.
Zoom in on the outer gas giants to see their moons.

## Controls

- **Pan:** right/middle mouse dragging pans in either mode. Switch to Pan mode
  with the tool button or **N** to pan using the left button or one-finger touch.
- **Zoom:** scroll at the cursor, use the panel's zoom buttons, or press + / -.
  **Home** restores a view of the initial system's area. Resizing preserves the
  camera's world center and zoom.
- **Create:** **Launch body** is selected at startup. Click a
  field and type to replace its value; Enter accepts, Tab advances, and Escape
  restores that field's previous value. Set mass, color as `#RRGGBB`, and X/Y
  velocity in world units per second (positive X right, positive Y down).
- **Launch:** click the world to place a body with the entered velocity, or drag
  from its spawn point in the desired direction. The arrow shows one second of
  initial travel, ignoring gravity; dragging updates the velocity readout and
  saves those values on release, rounded to two decimal places. The simulation
  temporarily pauses while aiming. Release over the panel or press Escape to
  cancel. Bodies can also be placed while manually paused.
- **Pause:** Space or the panel button. Losing focus pauses automatically.
  **P** toggles the panel; **Escape** cancels editing/dragging, otherwise closes.
- **Reset system:** restores the initial bodies and camera, discarding additions.

The panel supports touch buttons; editing numeric/color fields requires a
keyboard. Mass is limited to 0.01–100000, each velocity component to ±10000,
and the scene to 256 bodies to bound the cost of pairwise gravity.

Physics uses 240 fixed RK4 steps per second, with at most 100 ms of catch-up
following a stall. State and force calculations use `f64`; `phy` supplies `f32`
time steps and solver coefficients. Gravity now depends on the attracting body's
mass, using `a = G * other_mass * delta / (distance² + softening²)^(3/2)`, with
`G = 120` and softening length 8. This preserves equal and opposite pair forces
and stays finite at zero separation. Visual radii grow with the cube root of
mass; bodies can pass through one another and do not merge or collide.

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
cover mass-dependent forces, momentum conservation, three minutes of planetary
and moon orbits, body creation and validation, precision, trail trimming, time
accumulation, field editing, launch velocities, and camera transforms.
