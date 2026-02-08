# Tower Oops!

A modern GTK4 port of the classic "Crazy Towers", rewritten in Rust.

Summary: a small tower/match puzzle game.

Fully ported to Rust and modularized for clarity and extension. Multiple AI difficulty levels (including Minimax with alpha-beta pruning).

![Screenshot](screenshot.png)

## Build & Run

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

## Controls

- Click a cell to select or place towers.
- Use the menu to start a new game, change who begins, or open settings.

## Repository layout

- `src/` — Rust source
- `resources/` — Graphics and other asset files (SVG preferred)
- `Cargo.toml` — Cargo manifest

## Contributing

Contributions are welcome. For changes that affect performance or resource handling where appropriate. Please open pull requests against `main`.

Pay attention. Code is created by "vibe coding".

## Credits, links and licenses

- Project source code: MIT (see `LICENSE`).
- Graphic assets referenced in this repo are public-domain (e.g. AI generated) / CC0 where noted.

Related links:

- Crazy Towers (inspiration): https://www.crazybytes.at/games/games_free_D.htm#towers

Example asset credits (provided for transparency):

- Banana icon (CC0): https://commons.wikimedia.org/wiki/File:Banana_icon.svg
- Bomb icon (CC0): https://commons.wikimedia.org/wiki/File:Bomb-156107.svg

----

Enjoy! 🎮
