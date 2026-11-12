import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_clippy


class ClippyBudgetTests(unittest.TestCase):
    def check(self, messages, baseline, exit_code=0):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "baseline.json"
            path.write_text(json.dumps(baseline))
            with patch.object(check_clippy.subprocess, "Popen") as start:
                start.return_value.stdout = [json.dumps(item) for item in messages]
                start.return_value.wait.return_value = exit_code
                with patch("sys.argv", ["check_clippy.py", "--baseline", str(path)]):
                    with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                        return check_clippy.main()

    def warning(self, line=1):
        return {"reason": "compiler-message", "message": {
            "level": "warning", "message": "example", "code": {"code": "clippy::example"},
            "spans": [{"file_name": "src/example.rs", "line_start": line, "column_start": 1, "is_primary": True}],
        }}

    def test_shared_binary_and_test_warning_is_counted_once(self):
        self.assertEqual(self.check([self.warning(), self.warning()], {"clippy::example": 1}), 0)

    def test_new_warning_exceeds_budget(self):
        self.assertEqual(self.check([self.warning()], {}), 1)

    def test_increased_count_exceeds_budget(self):
        self.assertEqual(self.check([self.warning(1), self.warning(2)], {"clippy::example": 1}), 1)

    def test_compiler_failure_cannot_pass_an_empty_budget(self):
        self.assertEqual(self.check([], {}, exit_code=1), 1)


if __name__ == "__main__":
    unittest.main()
