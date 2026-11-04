import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("prepare_release_notes", ROOT / "tools/prepare_release_notes.py")
notes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notes)


class ReleaseNotesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        (self.root / "source.txt").write_text("initial", encoding="utf-8")
        self.commit("initial release")
        self.git("tag", "v0.7.1-alpha")

    def tearDown(self):
        self.temp.cleanup()

    def git(self, *args):
        subprocess.run(["git", *args], cwd=self.root, check=True, capture_output=True)

    def commit(self, subject):
        self.git("add", ".")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", subject)

    def test_next_minor_preserves_prerelease_stage(self):
        self.assertEqual(notes.next_tag("v0.7.1-alpha"), "v0.8.0-alpha")
        self.assertEqual(notes.next_tag("v1.9.3"), "v1.10.0")
        self.assertEqual(notes.next_tag("v1.9.3-rc.2"), "v1.10.0-rc")
        with self.assertRaises(ValueError):
            notes.next_tag("not-a-version")

    def test_commits_are_updated_without_overwriting_manual_notes(self):
        path = notes.prepare("v0.7.1-alpha", self.root)
        self.assertEqual(path.name, "v0.8.0-alpha.md")
        path.write_text(path.read_text().replace("Add user-facing highlights here.", "A manually written highlight."))
        (self.root / "source.txt").write_text("changed", encoding="utf-8")
        self.commit("feat: add media conversion")
        notes.prepare("v0.7.1-alpha", self.root)
        content = path.read_text()
        self.assertIn("A manually written highlight.", content)
        self.assertIn("feat: add media conversion", content)
        self.assertNotIn("No new commits", content)
        self.commit("chore(release-notes): update upcoming release notes")
        notes.prepare("v0.7.1-alpha", self.root)
        self.assertNotIn("chore(release-notes):", path.read_text())

    def test_existing_unmanaged_notes_are_not_overwritten(self):
        path = self.root / "release-notes" / "v0.8.0-alpha.md"
        path.parent.mkdir()
        path.write_text("My release notes", encoding="utf-8")
        with self.assertRaises(ValueError):
            notes.prepare("v0.7.1-alpha", self.root)
        self.assertEqual(path.read_text(), "My release notes")


if __name__ == "__main__":
    unittest.main()
