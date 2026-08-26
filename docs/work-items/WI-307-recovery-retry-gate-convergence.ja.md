---
author: AI Cockpit maintainers
title: "Recovery retry projection gate の収束"
description: "static governance-integrity gate を stale retry evidence の Runtime 消費規則へ整合させる。"
audience:
  - maintainer
  - reviewer
workItemId: WI-307-recovery-retry-gate-convergence
status: implemented
lastVerifiedBy: WI-307-recovery-retry-gate-convergence
terminalArchive: .ai/work-items/archive/WI-307-recovery-retry-gate-convergence.contract.json
terminalVerification: .ai/evidence/WI-307-recovery-retry-gate-convergence.verification.json
terminalFinalization: .ai/decisions/WI-307-recovery-retry-gate-convergence.finalize.45784a2d6fa2092944e6e238cb7b05755f4f7a30aab55c317032d6b81207da36.json
terminalDecision: .ai/decisions/WI-307-recovery-retry-gate-convergence.close.json
authority: canonical
---

# Recovery retry projection gate の収束

## Intent と goal

WI-306 で CI に限定された不整合が判明しました。Rust Runtime は fresh verification
によって predecessor Contract/Summary/Outcome/Events binding が進んだ後、古い
retry を projection しません。しかし static governance-integrity gate は一時的な
blocked Summary marker を要求し、ない場合に retry を現在の recovered terminal と
誤認していました。本 Work Item は fail-closed recovery を弱めず、歴史 bytes を
書き換えずに gate と Runtime の identity 規則を一致させます。

## Scope と source

- `tests/ci/governance_integrity_gate.py`
- `tests/ci/governance_integrity_gate_test.sh`
- `tests/ci/fixtures/governance-integrity`
- 三言語の Agent workflow / command reference

source は installed Rust Runtime の recovery read-side validation
（`load_recovery_decision`）と、fresh verification 後も WI-306 が
`docs_governance_integrity` / `missing_parity_decision` を報告した hosted run
`32978852886` です。

## Decisions

predecessor digest が valid で fresh archive と一致しなくなり、archive Outcome が
green の場合だけ gate は `retry` を消費します。predecessor digest のない legacy
fixture は明示的な blocked Summary compatibility path を使い続けます。invalid、
foreign、malformed、ambiguous、successor、supersede は従来どおり fail closed です。
gate は recovered terminal を作らず、実際の finalization path を projection します。

Rust Runtime protocol、repository archive、Outcome、verification、recovery bytes は
書き換えません。これは semantic alignment であり、source code や wire format の
コピーではありません。

## Acceptance と verification

- fresh green archive の stale retry が `finalize` と `awaiting_merge_close` を投影する;
- blocked のままの retry は recovery boundary として残る;
- successor/supersede、malformed/foreign candidate は fail closed を維持する;
- 三言語 workflow と command document が同じ規則を記載する;
- `bash tests/ci/governance_integrity_gate_test.sh`;
- `bash tests/ci/recovery_gate_acceptance.sh`;
- `bash tests/docs/documentation_acceptance.sh`;
- `cargo test --locked --workspace`。

## Boundary

外部 Runtime は共有し、repository state は分離したままです。本 Work Item は
repository の static CI projection と document のみを変更し、provider call、release
behavior、global Agent/MCP configuration は追加しません。
