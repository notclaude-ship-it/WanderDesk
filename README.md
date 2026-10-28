# WanderDesk

WanderDesk is an experimental, touch-friendly Linux application launcher that presents installed desktop applications as objects in a navigable 3D-style scene.

> **Status:** early development. The current scene is a custom 2D projection, not a 3D engine. The split-screen option is a stereoscopic mock-up, not OpenXR/VR support.

## Features

- Discovers visible applications from freedesktop `.desktop` entries.
- Walk around the scene with **WASD/ZQSD** and turn with **← / →**.
- Press **E** to attach/detach a nearby application and **F** to launch it.
- Press **1–9** to launch attached applications; **Esc** returns to the menu.
- On-screen movement/action controls work with mouse clicks and touch input.
- Full-screen startup is supported with `--fullscreen`.

## Run from source

Python 3.10+ and PyQt6 are required.

```bash
python3 -m venv .venv
. .venv/bin/activate
pip install -e '.[test]'
python wanderdesk.py
# or: python wanderdesk.py --fullscreen
```

On Debian/Ubuntu, system PyQt6 is also available via `sudo apt install python3-pyqt6`; run with the system Python in that case.

## Tests

```bash
python -m unittest discover -s tests -v
```

## Roadmap

1. Make application discovery and launching robust and testable.
2. Separate scene simulation, input handling, and rendering.
3. Add persistent settings and an application search/quick-launch panel.
4. Improve touch layouts for different screen sizes and add interaction tests.
5. Evaluate a real 3D renderer only after the launcher workflow is solid.

## Scope and safety

Applications are launched in the current desktop session. WanderDesk is not a Wayland compositor and does not currently provide an OpenXR runtime. Desktop `Exec` entries are parsed as launch metadata; they are never run during discovery.
