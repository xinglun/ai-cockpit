#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
parser.add_argument("--manifest", required=True)
parser.add_argument("--report", required=True)
parser.add_argument("--list-only", action="store_true")
args = parser.parse_args()
repo = Path(args.repo).resolve()
manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
gates = manifest["gates"]
results = []
failed = False
for gate in gates:
    result = {"category": gate["category"], "command": gate["command"], "id": gate["id"]}
    if gate.get("covers"):
        result["covers"] = gate["covers"]
    if args.list_only:
        result["state"] = "listed"
    else:
        command = list(gate["command"])
        if command[0].endswith(".sh"):
            command.insert(0, "bash")
        try:
            completed = subprocess.run(command, cwd=repo, check=False)
        except OSError as error:
            result["launchError"] = str(error)
            result["state"] = "failed"
            failed = True
        else:
            result["state"] = "passed" if completed.returncode == 0 else "failed"
            failed = failed or completed.returncode != 0
    results.append(result)
report = {
    "gates": results,
    "schemaVersion": 1,
    "state": "listed" if args.list_only else ("failed" if failed else "passed"),
}
target = Path(args.report)
target.parent.mkdir(parents=True, exist_ok=True)
target.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
raise SystemExit(1 if failed else 0)
