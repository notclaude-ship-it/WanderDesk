import tempfile
import unittest
from pathlib import Path

from app_catalog import discover_apps


class DiscoverAppsTests(unittest.TestCase):
    def write_desktop(self, root: Path, filename: str, content: str) -> None:
        (root / filename).write_text(content, encoding='utf-8')

    def test_filters_hidden_and_non_app_entries(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_desktop(root, 'visible.desktop', '[Desktop Entry]\nType=Application\nName=Visible\nExec=visible-app\n')
            self.write_desktop(root, 'hidden.desktop', '[Desktop Entry]\nType=Application\nName=Hidden\nExec=hidden-app\nHidden=true\n')
            self.write_desktop(root, 'nodisplay.desktop', '[Desktop Entry]\nType=Application\nName=Internal\nExec=internal-app\nNoDisplay=true\n')
            self.write_desktop(root, 'link.desktop', '[Desktop Entry]\nType=Link\nName=Website\nURL=https://example.com\n')
            self.assertEqual(discover_apps([root]), [('Visible', 'visible-app')])

    def test_ignores_missing_exec_and_malformed_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_desktop(root, 'noexec.desktop', '[Desktop Entry]\nType=Application\nName=No Exec\n')
            self.write_desktop(root, 'broken.desktop', 'this is not a desktop file')
            self.assertEqual(discover_apps([root]), [])

    def test_recursive_scan_sorting_and_limit(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            nested = root / 'nested'
            nested.mkdir()
            self.write_desktop(root, 'z.desktop', '[Desktop Entry]\nType=Application\nName=Zulu\nExec=zulu\n')
            self.write_desktop(nested, 'a.desktop', '[Desktop Entry]\nType=Application\nName=alpha\nExec=alpha\n')
            self.write_desktop(nested, 'b.desktop', '[Desktop Entry]\nType=Application\nName=Beta\nExec=beta\n')
            self.assertEqual([name for name, _ in discover_apps([root], limit=2)], ['alpha', 'Beta'])

    def test_missing_directory_and_negative_limit_are_safe(self):
        self.assertEqual(discover_apps(['/path/that/does/not/exist']), [])
        with tempfile.TemporaryDirectory() as tmp:
            self.write_desktop(Path(tmp), 'a.desktop', '[Desktop Entry]\nType=Application\nName=A\nExec=a\n')
            self.assertEqual(discover_apps([tmp], limit=-1), [])


if __name__ == '__main__':
    unittest.main()
