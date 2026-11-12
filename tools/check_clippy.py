"""Fail on compiler errors or growth in the recorded warning budget by lint."""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", action="store_true", help="Print current counts as JSON; does not edit the baseline")
    parser.add_argument("--baseline", type=Path, default=Path(__file__).with_name("clippy-baseline.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    process = subprocess.Popen(
        [os.environ.get("CARGO", "cargo"), "clippy", "--all-targets", "--locked", "--message-format=json"],
        cwd=root, stdout=subprocess.PIPE, text=True,
    )
    warnings = set()
    counts = Counter()
    for line in process.stdout:
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if item.get("reason") != "compiler-message":
            continue
        message = item["message"]
        if message["level"] not in ("warning", "error"):
            continue
        rendered = message.get("rendered")
        if message["level"] == "error" and rendered:
            print(rendered, file=sys.stderr)
        if message["level"] != "warning":
            continue
        code = (message.get("code") or {}).get("code", "compiler-warning")
        spans = tuple((s["file_name"], s["line_start"], s["column_start"]) for s in message["spans"] if s["is_primary"])
        identity = (code, message["message"], spans)
        if identity not in warnings:
            warnings.add(identity)
            counts[code] += 1
    if process.wait() != 0:
        return 1
    if args.record:
        print(json.dumps(dict(sorted(counts.items())), indent=2))
        return 0
    baseline = json.loads(args.baseline.read_text())
    increased = {code: count for code, count in counts.items() if count > baseline.get(code, 0)}
    if increased:
        for code, count in sorted(increased.items()):
            print(f"{code}: {count} warnings, budget {baseline.get(code, 0)}", file=sys.stderr)
        print("Fix the new warnings, or review an explicit baseline update. Run cargo clippy for locations.", file=sys.stderr)
        return 1
    print(f"Clippy budget passed: {sum(counts.values())} unique warnings (budget {sum(baseline.values())}).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
