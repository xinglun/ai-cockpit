#!/usr/bin/env python3
"""Mandatory Linux predecessor check using one source-bound dev fixture asset.

The CI builder is the trusted producer. This is not independent attestation
against a producer that can replace both the binary and its provenance.
No Cargo process or asset download is started by this consumer.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import platform
import signal
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


SOURCE = "dd2339bee3dc63e5ce7547d2aa950e315088b1fe"
TREE = "4c59ecb41933be132cb2c887082d86eb17355169"
LOCK = "76b02d3cf1e9d5944c8f62b142b0082f40f7493ce92d527bcee5de7ff66310d7"
TARGET = "x86_64-unknown-linux-gnu"
VERSION = "ai-cockpit 1.0.1"
BUILD = {"profile": "dev", "debug": 0, "incremental": False, "jobs": 1,
         "target": TARGET, "rustflags": "-D warnings", "rustdocflags": "-D warnings"}
ID = "WI-EVIDENCE-STAGES"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def regular_bytes(path: Path, limit: int) -> bytes:
    path = path.absolute()
    require(not any(p.is_symlink() for p in (path, *path.parents)),
            f"symbolic link is not a fixture asset: {path}")
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
                         | getattr(os, "O_NONBLOCK", 0))
    try:
        info = os.fstat(descriptor)
        require(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= limit,
                f"complete bounded regular fixture file required: {path}")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            raw = stream.read(limit + 1)
        require(len(raw) == info.st_size and len(raw) <= limit,
                f"fixture file changed or exceeded its bound: {path}")
        return raw
    finally:
        os.close(descriptor)


def validate_asset(binary: Path, provenance: Path) -> dict:
    raw = regular_bytes(binary, 256 * 1024 * 1024)
    value = json.loads(regular_bytes(provenance, 64 * 1024))
    require(isinstance(value, dict) and set(value) == {
        "schemaVersion", "kind", "sourceCommit", "sourceTree", "cargoLockSha256",
        "toolchain", "build", "binary"}, "unexpected predecessor provenance fields")
    require(type(value["schemaVersion"]) is int and value["schemaVersion"] == 1
            and value["kind"] == "evidence_stage_predecessor",
            "unsupported predecessor provenance")
    require(value["sourceCommit"] == SOURCE and value["sourceTree"] == TREE
            and value["cargoLockSha256"] == LOCK, "wrong predecessor source identity")
    require(isinstance(value["build"], dict) and value["build"] == BUILD
            and all(type(value["build"][key]) is type(expected)
                    for key, expected in BUILD.items()), "wrong predecessor build parameters")
    toolchain = value["toolchain"]
    require(isinstance(toolchain, str)
            and toolchain.splitlines()
            and toolchain.splitlines()[0] == "rustc 1.98.1 (48a229cea 2026-09-01)"
            and f"host: {TARGET}" in toolchain.splitlines(), "wrong predecessor toolchain")
    binary_info = value["binary"]
    require(isinstance(binary_info, dict)
            and binary_info == {"sha256": digest(raw), "bytes": len(raw), "version": VERSION},
            "predecessor binary identity mismatch")
    require(os.access(binary, os.X_OK), "predecessor binary is not executable")
    return value


def run(argv: list[str], cwd: Path, timeout: int = 20) -> subprocess.CompletedProcess:
    environment = os.environ.copy()
    count = int(environment.get("GIT_CONFIG_COUNT", "0"))
    for index, key in enumerate(("maintenance.autoDetach", "gc.autoDetach"), count):
        environment[f"GIT_CONFIG_KEY_{index}"] = key
        environment[f"GIT_CONFIG_VALUE_{index}"] = "false"
    environment["GIT_CONFIG_COUNT"] = str(count + 2)
    process = subprocess.Popen(argv, cwd=cwd, env=environment,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=True)
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except BaseException:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.communicate(timeout=5)
        raise
    require(len(stdout) <= 64 * 1024 and len(stderr) <= 64 * 1024,
            "compatibility command output exceeded its complete-capture bound")
    return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)


def successful(argv: list[str], cwd: Path) -> bytes:
    result = run(argv, cwd)
    require(result.returncode == 0,
            f"fixture setup failed: {argv!r}: {result.stderr.decode('utf-8', 'replace')}")
    return result.stdout


def acceptance(binary: Path, legacy: Path, provenance: Path) -> dict:
    require(sys.platform == "linux" and platform.machine() == "x86_64",
            "this predecessor fixture acceptance requires Linux x86_64")
    asset = validate_asset(legacy, provenance)
    candidate_digest = digest(regular_bytes(binary, 256 * 1024 * 1024))
    require(candidate_digest != asset["binary"]["sha256"],
            "candidate and predecessor must be distinct binaries")
    with tempfile.TemporaryDirectory(prefix="evidence stage compatibility ") as temporary:
        root = Path(temporary).resolve()
        require(successful([str(legacy), "--version"], root).decode().strip() == VERSION,
                "predecessor version differs from the producer record")
        for args in (("init", "-q", "-b", "main"), ("config", "core.autocrlf", "false"),
                     ("config", "user.name", "Compatibility Fixture"),
                     ("config", "user.email", "fixture@example.invalid")):
            successful(["git", *args], root)
        (root / "tracked.txt").write_bytes(b"stage compatibility fixture\n")
        for suffix in ("", ".zh-CN", ".ja"):
            page = root / f"docs/work-items/{ID}{suffix}.md"
            page.parent.mkdir(parents=True, exist_ok=True)
            page.write_text(f"---\nstatus: in_progress\nworkItemId: {ID}\nlastVerifiedBy: {ID}\n---\n\n# Isolated stage fixture\n", encoding="utf-8")
            ledger = root / f"docs/reference/reference-parity{suffix}.md"
            ledger.parent.mkdir(parents=True, exist_ok=True)
            ledger.write_text(f"# Reference parity\n\n| {ID} | In progress | [Work Item](../work-items/{ID}{suffix}.md) | planned terminal lifecycle: archive `.ai/work-items/archive/{ID}.contract.json`; verification `.ai/evidence/{ID}.verification.json`; close `.ai/decisions/{ID}.close.json` |\n", encoding="utf-8")
        successful(["git", "add", "."], root)
        successful(["git", "commit", "-qm", "stage fixture baseline"], root)
        successful([str(binary), "attach", "--repo", str(root)], root)
        successful([str(binary), "start", "--repo", str(root), "--id", ID,
                    "--intent", "allow source validation before final adoption",
                    "--goal", "preserve final evidence gates", "--scope", "**",
                    "--authority", "authorized", "--risk", "low",
                    "--acceptance", "final adoption remains required",
                    "--required-evidence", "verification,adopter",
                    "--verification", "git diff --check"], root)
        contract = root / f".ai/work-items/active/{ID}.contract.json"
        summary = root / f".ai/work-items/active/{ID}.summary.json"
        before = json.loads(contract.read_bytes())
        stages = {"schemaVersion": 1, "classes": [{"class": "adopter", "requiredFrom": "finish"}]}
        request = {"schemaVersion": 1, "changeId": "staged-evidence-fixture",
                   "expectedContractDigest": "sha256:" + digest(json.dumps(before, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()),
                   "reason": "explicit final evidence boundary in isolated test fixture",
                   "changes": [{"path": "/governanceProfile", "operation": "set",
                                "value": {"evidenceClassStages": stages}}]}
        amendment = root / "amendment-input.json"
        amendment.write_text(json.dumps(request), encoding="utf-8")
        successful([str(binary), "work-item", "amend", "--repo", str(root),
                    "--id", ID, "--request", str(amendment)], root)
        controls = root / "controls-input.json"
        controls.write_text(json.dumps({"intentAlignment": {"state": "resolved", "evidence": ["tracked.txt"]}}), encoding="utf-8")
        successful([str(binary), "work-item", "controls", "--repo", str(root),
                    "--id", ID, "--input", str(controls)], root)
        amended = json.loads(contract.read_bytes())
        require(amended["governanceProfile"]["evidenceClassStages"] == stages
                and "evidence-class-stages" in amended["requiredRuntimeCapabilities"],
                "native amendment did not establish the protected capability")
        contract_before, summary_before = contract.read_bytes(), summary.read_bytes()
        result = run([str(legacy), "work-item", "status", "--repo", str(root),
                      "--id", ID, "--json"], root)
        stderr = result.stderr.decode("utf-8", "strict")
        require(result.returncode != 0, "actual predecessor accepted the new capability")
        require("requiredRuntimeCapabilities contains unsupported capability" in stderr
                and "evidence-class-stages" in stderr, "wrong predecessor rejection reason")
        require(contract.read_bytes() == contract_before, "predecessor modified Contract bytes")
        require(summary.read_bytes() == summary_before, "predecessor modified Summary bytes")
        receipt = root / f".ai/evidence/{ID}.verification.json"
        require(not receipt.exists() and not receipt.is_symlink(),
                "predecessor published an unexpected verification receipt")
        require(validate_asset(legacy, provenance) == asset, "predecessor fixture changed during execution")
        require(digest(regular_bytes(binary, 256 * 1024 * 1024)) == candidate_digest,
                "candidate changed during execution")
        return {"state": "passed", "kind": "evidence_stage_runtime_compatibility",
                "predecessor": asset, "candidateSha256": candidate_digest,
                "exitCode": result.returncode, "stdout": result.stdout.decode("utf-8", "strict"),
                "stderr": stderr, "contractSha256": digest(contract_before),
                "summarySha256": digest(summary_before), "verificationReceiptCreated": False}


def self_test() -> None:
    """Exercise asset rejection only; these fixtures are never executed."""
    with tempfile.TemporaryDirectory(prefix="predecessor asset validators ") as temporary:
        root = Path(temporary)
        binary, provenance = root / "asset", root / "provenance.json"
        raw = b"nonexecuted asset validation fixture\n"
        binary.write_bytes(raw)
        binary.chmod(0o700)
        value = {"schemaVersion": 1, "kind": "evidence_stage_predecessor",
                 "sourceCommit": SOURCE, "sourceTree": TREE, "cargoLockSha256": LOCK,
                 "toolchain": f"rustc 1.98.1 (48a229cea 2026-09-01)\nhost: {TARGET}\n",
                 "build": BUILD, "binary": {"sha256": digest(raw), "bytes": len(raw), "version": VERSION}}
        def publish(item: dict) -> None:
            provenance.write_text(json.dumps(item), encoding="utf-8")
        publish(value)
        require(validate_asset(binary, provenance) == value, "valid asset fixture rejected")
        cases = []
        for key in ("sourceCommit", "sourceTree", "cargoLockSha256", "toolchain", "kind"):
            changed = copy.deepcopy(value); changed[key] = "wrong"; cases.append((key, changed))
        for key, replacement in (("profile", "release"), ("debug", 1), ("debug", False), ("incremental", True),
                                 ("jobs", 2), ("target", "aarch64-apple-darwin"),
                                 ("rustflags", ""), ("rustdocflags", "")):
            changed = copy.deepcopy(value); changed["build"][key] = replacement; cases.append((key, changed))
        for key, replacement in (("sha256", "0" * 64), ("bytes", len(raw) + 1), ("version", "wrong")):
            changed = copy.deepcopy(value); changed["binary"][key] = replacement; cases.append((key, changed))
        for name, changed in cases:
            publish(changed)
            try:
                validate_asset(binary, provenance)
            except (ValueError, OSError):
                continue
            raise ValueError(f"changed asset {name} was accepted")
        publish(value)
        for name in ("missing", "symlink", "fifo", "not-executable", "tampered"):
            binary.unlink()
            if name == "symlink": binary.symlink_to(provenance)
            elif name == "fifo": os.mkfifo(binary)
            elif name == "not-executable": binary.write_bytes(raw); binary.chmod(0o600)
            elif name == "tampered": binary.write_bytes(raw + b"changed"); binary.chmod(0o700)
            try:
                validate_asset(binary, provenance)
            except (ValueError, OSError):
                if binary.exists() or binary.is_symlink(): binary.unlink()
                binary.write_bytes(raw); binary.chmod(0o700)
                continue
            raise ValueError(f"{name} fixture asset was accepted")
        for name in ("missing", "symlink", "malformed", "empty", "unknown-field"):
            provenance.unlink()
            if name == "symlink": provenance.symlink_to(binary)
            elif name == "malformed": provenance.write_bytes(b"{")
            elif name == "empty": provenance.write_bytes(b"")
            elif name == "unknown-field":
                changed = copy.deepcopy(value); changed["extra"] = True; publish(changed)
            try:
                validate_asset(binary, provenance)
            except (ValueError, OSError):
                if provenance.exists() or provenance.is_symlink(): provenance.unlink()
                publish(value)
                continue
            raise ValueError(f"{name} provenance was accepted")
    print(json.dumps({"state": "passed", "assetValidationCases": 1 + len(cases) + 10,
                      "runtimeAcceptanceExecuted": False}))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--legacy-binary", type=Path)
    parser.add_argument("--legacy-provenance", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            require(args.binary is None and args.legacy_binary is None and args.legacy_provenance is None,
                    "asset self-test is separate from actual Runtime acceptance")
            self_test()
        else:
            require(all(p is not None for p in (args.binary, args.legacy_binary, args.legacy_provenance)),
                    "--binary, --legacy-binary and --legacy-provenance are required")
            print(json.dumps(acceptance(args.binary.absolute(), args.legacy_binary.absolute(),
                                        args.legacy_provenance.absolute()), sort_keys=True))
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(json.dumps({"state": "failed", "diagnostic": {
            "phase": "evidence_stage_runtime_compatibility", "message": str(error)}}), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
