# Tower Oops!

A modern GTK4 port of the classic "Crazy Towers", rewritten in Rust.

Summary: a small tower/match puzzle game with scalable, crisp SVG graphics,
configurable animations, and adjustable AI difficulty.

Features:
- Fully ported to Rust and modularized for clarity and extension.
- Scalable UI: artwork scales proportionally to the window size.
- SVG support: vector assets are rendered via `resvg` → `tiny-skia` and
	rasterized at device-pixel resolution so they remain sharp at any size.
- Animated selection pulse and smooth tower animations.
- Multiple AI difficulty levels (including Minimax with alpha-beta pruning).

Important: artwork and icons live in the `resources/` directory.

Build & Run

Prerequisites (Debian/Ubuntu):
```bash
sudo apt install build-essential libgtk-4-dev pkg-config
```

Rust and Cargo (via `rustup`) are required. Dependencies are managed by Cargo.

To build and run a release build:
```bash
cargo build --release
cargo run --release
```

For iterative development, use:
```bash
cargo run
```

HiDPI / Sharpness

SVG assets are rasterized at the needed pixel resolution. On HiDPI systems
or when forcing higher rasterization, you can set environment variables before
starting the app:

```bash
# Example: force 2x rasterization
export GDK_SCALE=2
export GDK_DPI_SCALE=1
cargo run --release
```

The app caches rasterized pixmaps for performance. If you use multiple
monitors with different DPI settings, consider starting the app on the
intended monitor or extending the monitor API integration.

Repository Layout (key parts)
- `src/` — Rust source code
- `resources/` — Graphics: prefer `.svg`, `.png` used as fallbacks
- `Cargo.toml` — Cargo manifest and dependencies

Usage / Controls
- Click a cell to start a selection — the selected cell pulses briefly as
	visual feedback.
- Use the menu to start a new game, toggle who begins, or open settings.

Credits & Graphic Licenses
The following SVG images were obtained from Wikimedia Commons and are CC0:

- Banana icon (CC0): https://commons.wikimedia.org/wiki/File:Banana_icon.svg
- Bomb icon (CC0): https://commons.wikimedia.org/wiki/File:Bomb-156107.svg

Links
- Based on Crazy Towers: https://www.crazybytes.at/games/games_free_D.htm#towers

Note: project source code is licensed under MIT (see `LICENSE`). The graphic
assets listed above are CC0 (public domain). Attribution is not legally
required but is provided here for transparency.

Contributing

Pull requests are welcome. For performance changes, please preserve or
improve the SVG raster cache strategy and include tests or benchmarks when
possible.


Enjoy! 🎮
