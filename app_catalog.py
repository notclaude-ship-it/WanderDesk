"""Discovery of launchable applications from freedesktop .desktop files."""
from __future__ import annotations

import configparser
import os
from pathlib import Path
import shutil


def discover_apps(directories: list[str | Path] | None = None, limit: int = 90) -> list[tuple[str, str]]:
    """Return sorted ``(display_name, exec_line)`` pairs for visible applications.

    Desktop files are read as data, not executed. Hidden, NoDisplay, non-application,
    and entries without Exec are ignored. Duplicate desktop-file names are avoided
    while allowing two distinct apps to share a display name.
    """
    if directories is None:
        directories = [Path('/usr/share/applications'), Path.home() / '.local/share/applications']
    found: dict[str, tuple[str, str]] = {}
    for folder in directories:
        root = Path(folder).expanduser()
        if not root.is_dir():
            continue
        for path in sorted(root.rglob('*.desktop')):
            parser = configparser.ConfigParser(interpolation=None, strict=False)
            parser.optionxform = str
            try:
                parser.read(path, encoding='utf-8')
                entry = parser['Desktop Entry']
                if entry.get('Type', 'Application') != 'Application':
                    continue
                if entry.getboolean('Hidden', fallback=False) or entry.getboolean('NoDisplay', fallback=False):
                    continue
                name = entry.get('Name', path.stem).strip()
                command = entry.get('Exec', '').strip()
                if not name or not command:
                    continue
                try_exec = entry.get('TryExec', '').strip()
                if try_exec and os.path.sep not in try_exec and shutil.which(try_exec) is None:
                    continue
                elif try_exec and os.path.sep in try_exec and not Path(try_exec).exists():
                    continue
                # Relative path is part of the key so user overrides can coexist.
                key = str(path.resolve())
                found[key] = (name, command)
            except (OSError, KeyError, ValueError, configparser.Error, UnicodeError):
                continue
    return sorted(found.values(), key=lambda app: app[0].casefold())[:max(0, limit)]
