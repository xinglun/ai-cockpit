#!/usr/bin/env python3
"""Equivalent isolated fixture, never real Sentinel evidence or authority."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
from datetime import datetime, timezone

signal.alarm(890)
base = Path(__file__).parent
case = base / sys.argv[1]
case.mkdir(exist_ok=False)
root = case / "repository"
root.mkdir()
logs = case / "logs"
logs.mkdir()
binary = Path("/workspace/ai-cockpit-wi1089-recovery/target/release/ai-cockpit")
assert hashlib.sha256(binary.read_bytes()).hexdigest() == "d9d1fc38e275f53e9541f7c7bd9f8b13bdae3269a904922dc3d4cb50ae8cb18a"
wid = "WI-RETAINED-CANONICAL-FIXTURE"
env = os.environ.copy()
env.update(PATH="/workspace/.cloud-tools/cargo/bin:" + env["PATH"],
           CARGO_HOME="/workspace/.cloud-tools/cargo", RUSTUP_HOME="/workspace/.cloud-tools/rustup",
           CARGO_TARGET_DIR=str(root / ".ai/fixture-target"), CARGO_BUILD_JOBS="1",
           CARGO_INCREMENTAL="0", CARGO_NET_OFFLINE="true", RUST_TEST_THREADS="1")
number = 0

def run(label, argv, expect=0):
    global number
    number += 1
    prefix = logs / f"{number:02}-{label}"
    started = datetime.now(timezone.utc).isoformat()
    result = subprocess.run([str(a) for a in argv], cwd=root, env=env,
                            capture_output=True, timeout=600)
    prefix.with_suffix(".stdout").write_bytes(result.stdout)
    prefix.with_suffix(".stderr").write_bytes(result.stderr)
    prefix.with_suffix(".metadata.json").write_text(json.dumps({
        "argv": [str(a) for a in argv], "exitCode": result.returncode,
        "startedAt": started, "endedAt": datetime.now(timezone.utc).isoformat(),
        "stdoutSha256": hashlib.sha256(result.stdout).hexdigest(),
        "stderrSha256": hashlib.sha256(result.stderr).hexdigest()}, indent=2) + "\n")
    print(f"{number:02} {label}: exit {result.returncode}", flush=True)
    if expect is not None and result.returncode != expect:
        raise AssertionError(f"{label}: {result.stderr.decode()}\n{result.stdout.decode()}")
    return result

def cli(label, args, expect=0):
    return run(label, [binary] + args + ["--repo", root], expect)

def document(suffix):
    return json.loads((root / f".ai/work-items/active/{wid}.{suffix}.json").read_text())

def amendment(label, changes):
    contract = document("contract")
    digest = "sha256:" + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
    request = {"schemaVersion": 1, "changeId": label, "expectedContractDigest": digest,
               "reason": "Isolated equivalent fixture, no real human or production authority.", "changes": changes}
    path = case / f"{label}.input.json"
    path.write_text(json.dumps(request, indent=2))
    return cli(label, ["work-item", "amend", "--id", wid, "--request", path])

def controls(label, value):
    path = case / f"{label}.input.json"
    path.write_text(json.dumps(value, indent=2))
    return cli(label, ["work-item", "controls", "--id", wid, "--input", path])

def review(label):
    summary = document("summary")
    return controls(label, {"decisionEvidence": {
        "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
        "workItemId": wid, "repositoryId": document("contract")["repositoryId"],
        "contractDigest": summary["preflightContractDigest"],
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "recordedBy": "human:isolated-synthetic-fixture",
        "reason": "Synthetic fixture decision only; not Ray or a consumer approval."}})

def material(label, expect=0):
    plan = cli(label + "-plan", ["work-item", "material-review", "plan", "--id", wid])
    request = json.loads(plan.stdout)
    assert not request["findingCodes"] and request["reviewEnabled"], request
    assert any(e.get("reviewable") for e in request["entries"]), request
    path = case / f"{label}.input.json"
    path.write_text(json.dumps({"schemaVersion": 1, "decision": "accept_permitted_unknowns",
        "requestDigest": request["requestDigest"], "reviewerActor": "agent:isolated-fixture-reviewer",
        "authoritySource": "test-fixture:bounded-review", "assurance": "self_declared",
        "evidenceRefs": [{"path": "tests/material.rs", "digest": "sha256:" + hashlib.sha256((root / "tests/material.rs").read_bytes()).hexdigest()}],
        "rationale": "Read the exact fixture source; only the bounded committed syntax Unknown.",
        "residualRisk": "The scanner is incomplete; synthetic fixture scope only."}, indent=2))
    result = cli(label + "-record", ["work-item", "material-review", "record", "--id", wid, "--input", path], expect)
    return request, result

run("git-init", ["git", "init", "-q", "--initial-branch=main"])
run("git-name", ["git", "config", "user.name", "Isolated Fixture"])
run("git-email", ["git", "config", "user.email", "fixture@example.invalid"])
(root / "Cargo.toml").write_text('[package]\nname = "retained_fixture"\nversion = "0.1.0"\nedition = "2024"\n')
(root / "src").mkdir()
(root / "src/lib.rs").write_text('pub fn expected_date() -> &\'static str {\n    "2026-10-14"\n}\n')
(root / "README.md").write_text("Equivalent lifecycle fixture, not Sentinel.\n")
run("generate-lockfile", ["cargo", "generate-lockfile", "--offline"])
run("git-base-add", ["git", "add", "."])
run("git-base-commit", ["git", "commit", "-qm", "fixture baseline"])
cli("attach", ["attach"])
commands = ["cargo test --locked --offline --test material", "cargo fmt --all -- --check",
            "cargo test --workspace --locked --offline", "cargo clippy --locked --offline --all-targets -- -D warnings"]
args = ["start", "--id", wid, "--intent", "Exercise preserved canonical recovery",
        "--goal", "Run four checks through fixture finish and archive", "--risk", "normal",
        "--authority", "authorized", "--out-of-scope", "No real consumer or human authority",
        "--acceptance", "All four commands execute and history remains intact",
        "--required-evidence", "verification_receipt"]
for path in ["README.md", "Cargo.toml", "Cargo.lock", "src/lib.rs", "tests/material.rs"]:
    args += ["--scope", path]
for command in commands:
    args += ["--verification", command]
cli("start", args)
scenario = {"scenario": "old fixture four-check cycle", "required": True, "status": "unverified",
            "evidence": [], "expected": "The old cycle executes all four declared checks.",
            "verificationPlan": "Native verify all four declared fixture commands."}
amendment("fixture-profile-before-checkpoint", [
    {"path": "/governanceProfile", "operation": "set", "value": {"materialInspectionReview": {
        "schemaVersion": 1, "permittedUnknown": "repository_material_inspection_unavailable",
        "permittedCause": "readable_committed_rust_syntax_unknown", "assurance": "self_declared",
        "reviewerActor": "agent:isolated-fixture-reviewer", "authoritySource": "test-fixture:bounded-review",
        "acceptResidualRisk": True}}},
    {"path": "/scenarioCoverage", "operation": "add", "value": scenario}])
relative = f".ai/work-items/active/{wid}.contract.json"
cli("initial-preflight", ["preflight", "--contract", relative])
review("initial-fixture-review")
controls("planned-fixture-scenario", {"scenarioCoverage": [scenario]})
cli("checkpoint", ["checkpoint", "--id", wid])
(root / "tests").mkdir()
(root / "tests/material.rs").write_text('''fn material() -> String {
    let mut label = String::from("token");
    label.push_str("ization");
    label
}

#[test]
fn composed_label_matches() {
    assert_eq!(material(), "tokenization");
    assert_eq!(retained_fixture::expected_date(), "2026-10-14");
}
''')
run("git-material-add", ["git", "add", "-A"])
run("git-material-commit", ["git", "commit", "-qm", "committed reviewable fixture source"])
cli("old-cycle-preflight", ["preflight", "--contract", relative])
review("old-cycle-fixture-review")
material("old-cycle-material")
cli("old-four-check-verify", ["verify", "--work-item", wid, "--workers", "1"])
evidence_path = root / f".ai/evidence/{wid}.verification.json"
old_bytes = evidence_path.read_bytes()
old = json.loads(old_bytes)
assert old["passed"] and old["evidenceSchemaVersion"] == 2
assert old["receipt"]["nodesExecuted"] == 4 and old["receipt"]["nodesReused"] == 0, old
assert old["receipt"]["processesSpawned"] == 4
assert all(r["passed"] for r in old["receipt"]["results"])
(case / "old-canonical.actual.json").write_bytes(old_bytes)
failure = cli("real-finish-blocked-by-planned-scenario", ["finish", "--id", wid, "--json"], None)
assert failure.returncode != 0, "fixture must create a real blocked Outcome before repair"
outcome = root / f".ai/work-items/active/{wid}.outcome.json"
assert json.loads(outcome.read_bytes())["state"] == "blocked"
(case / "old-blocked-outcome.actual.json").write_bytes(outcome.read_bytes())
preserved = root / ".ai/evidence/fixture-preserved-old-cycle.json"
preserved.write_bytes(old_bytes)
scenario = {**scenario, "status": "verified", "evidence": [".ai/evidence/fixture-preserved-old-cycle.json"]}
controls("old-cycle-scenario-evidence", {"scenarioCoverage": [scenario]})
amendment("sensitive-after-old-success-and-failed-finish", [
    {"path": "/acceptanceCriteria", "operation": "add", "value": "Reverify the exact amended plan before fixture finish and archive"}])
cli("amended-preflight", ["preflight", "--contract", relative])
review("amended-fixture-review")
status = json.loads(cli("status-before-material-recovery", ["work-item", "status", "--id", wid, "--json"]).stdout)
assert evidence_path.read_bytes() == old_bytes
assert status["sourceDigests"]["repositorySnapshot"] == old["repositorySnapshotDigest"], status
request, result = material("current-material-recovery", None)
plan = cli("current-verify-plan", ["verify", "--work-item", wid, "--workers", "1", "--plan-only"], None)
summary = {"kind": "equivalent_fixture_c960_retained_canonical_baseline_actual",
    "candidateSource": "c960ed2eaa9e42cddedad25612f5d0d2d9a44774", "candidateBinarySha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    "fixtureRoot": str(root), "oldCanonicalPresent": True, "oldCanonicalRawSha256": hashlib.sha256(old_bytes).hexdigest(),
    "oldCanonicalContract": old["contractDigest"], "currentContract": document("summary")["preflightContractDigest"],
    "oldSnapshot": old["repositorySnapshotDigest"], "currentSnapshot": status["sourceDigests"]["repositorySnapshot"],
    "oldActualNodesExecuted": 4, "oldActualNodesReused": 0, "oldActualProcessesSpawned": 4,
    "currentNativeStatus": status, "currentMaterialRequest": request["requestDigest"],
    "currentMaterialRecordExit": result.returncode, "currentVerifyPlanExit": plan.returncode,
    "limits": "Equivalent fixture, same producer version, native finish blocked by planned scenario; not the unpublished Sentinel receipt or its finish.preflight history."}
(case / "baseline.actual.json").write_text(json.dumps(summary, indent=2) + "\n")
assert result.returncode == 0, "RED expected: retained canonical c960 material recovery rejected; see baseline.actual.json and exact native stderr"
