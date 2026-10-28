# WanderDesk

WanderDesk is an experimental, touch-friendly Linux application world: installed desktop applications appear as objects in a navigable perspective scene.

## Native implementation

The primary development target is now **Rust + egui/eframe**. It produces a native Linux executable without requiring Python at runtime, uses the host's native windowing integration through winit, and supports both Wayland and X11. The same source can be compiled natively for **x86-64** and **AArch64/ARM64**.

The earlier Python/PyQt prototype remains in the repository temporarily for reference while native functionality is brought to parity.

> **Scope:** this is currently a graphical launcher running inside an existing Linux desktop session. It is not yet a Wayland compositor or a full desktop environment. The stereo setting is a placeholder; OpenXR/VR support is not implemented.

## Current features

- Finds visible applications from freedesktop `.desktop` entries in system and user application directories.
- Displays a perspective scene with application cards.
- Move with **WASD/ZQSD** (arrow up/down also move forward/back); turn with **← / →**.
- Press **E** to attach/detach the nearby app, **Enter/F** to launch it, and **1–9** to launch an attached app.
- Search the scene by application name.
- Session menu with movement speed, fullscreen, and refresh controls.
- Saves settings to `$XDG_CONFIG_HOME/wanderdesk/settings.toml` (or the default config directory).

## Build on Linux

Install Rust (stable) and the system development libraries required by winit/glutin. On Debian/Ubuntu:

```bash
sudo apt install build-essential pkg-config libwayland-dev libxkbcommon-dev \
  libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxrandr-dev libxcursor-dev libxi-dev \
  libxinerama-dev libxxf86vm-dev libegl1-mesa-dev libgl1-mesa-dev
```

Then build and run:

```bash
cargo test
cargo build --release
./target/release/wanderdesk
```

The release profile enables optimization, thin LTO, a single codegen unit, and stripping. The output is a native executable for the architecture on which it was built.

## Architectures and binaries

- **x86-64:** `cargo build --release` on an x86-64 Linux host.
- **ARM64/AArch64:** `cargo build --release` on an ARM64 Linux host.
- GitHub Actions builds and publishes separate CI artifacts for both architectures.

The two binaries are architecture-specific; an x86-64 executable cannot run natively on ARM64, or vice versa.

## Tests

```bash
cargo test
python3 -m unittest discover -s tests -v  # legacy Python prototype
```

## Roadmap

1. Finish native app selection, touch controls, icon rendering, and reliable freedesktop `Exec` handling.
2. Add integration tests and package artifacts as AppImage/deb packages.
3. Add proper 3D rendering and evaluate OpenXR.
4. Investigate a separate compositor architecture (for example, using Smithay) only after the launcher is stable.