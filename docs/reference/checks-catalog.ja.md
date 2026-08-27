---
author: AI Cockpit maintainers
title: "Checks Catalog"
description: "明示的な evidence boundary を持つ repository quality と governance checks。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, checks, governance, verification]
---

# Checks Catalog

この catalog は現在の Rust repository が実際に提供する check を説明します。source
の local quality check、Work Item governance gate、hosted provider evidence、enterprise
assurance の区別を保ちます。source の Make target や Python executor の copy では
ありません。

## Check の層

| 層 | Target entry point | 証明すること | 証明しないこと |
| --- | --- | --- | --- |
| Runtime Contract gate | `ai-cockpit gate --repo <path> --manifest tests/ci/repository_gate_manifest.json --stage <stage>` | 現在の Contract、repository snapshot、route、gate manifest の内部整合性。 | hosted CI を実行せず、enterprise assurance を付与しません。 |
| Local workspace quality | `cargo fmt --all -- --check`；`cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | checkout 済み workspace の Rust format と lint 結果。 | local pass は reviewed PR や Release の結果ではありません。 |
| Package verification | `tests/ci/run_workspace_package_tests.sh --report <path>` | deterministic package test coverage とその receipt。 | provider の branch protection や公開を証明しません。 |
| Conformance と documentation | `tests/conformance/reference_file_inventory_test.sh`；`tests/docs/documentation_acceptance.sh` | reference ledger、reader route、documentation invariant。 | 文書は executable evidence の代わりになりません。 |
| Release と adopter | strict manifest route の `tests/release/*` | artifact identity、checksum、SBOM/provenance binding、実行した named harness の isolated adopter lifecycle。 | staged/local result は provider receipt が明示しない限り public Release evidence ではありません。 |

Canonical set と profile floor は `tests/ci/repository_gate_manifest.json` で versioned
です。route は累積的です。`light` は documentation と低コスト policy check、`standard`
は Rust workspace と conformance、`strict` は release、workflow、performance、adopter
check を追加します。changed path、Contract risk、lifecycle stage が minimum profile を
選びます。unknown または release-owned input は `strict` に上がり、caller が速い command
を指定しても profile を下げられません。

`VerificationTier`（実行 check の強度）と `EvidenceAssurance`（結果を保証できる主体）は
直交します。strict の local checkだけで provider-verified や enterprise-verified には
なりません。

## Evidence の ownership

Runtime receipt は repository、Work Item、Contract、snapshot、selected route、Runtime
identity を bind します。Hosted CI は provider run/job conclusion と外部 branch/merge
observation を owner とします。Public Release は公開 archive、checksum、SBOM、provenance、
attestation の事実を owner とします。Enterprise system は identity、retention、WORM/SIEM、
organization approval を owner とします。AI Cockpit は delegated evidence を require、
bind、validate、display、archive できますが、外部 claim を forge しません。

すべての check は active Contract、preflight review、required scenario evidence、human
decision、reviewed PR lifecycle に従います。local green は有用な evidence ですが、gate
を省略したり production readiness を主張したりする authorization ではありません。

## Failure と recovery

Missing、malformed、stale、foreign、contradictory な receipt は fail closed です。失敗した
command、source revision、output receipt、provider run identity を diagnosis 用に保存します。
bounded cause を直して named check を rerun し、unpinned command や source-built Runtime で
失敗結果を置き換えません。object engineering adopter は自身の stack command を提供し、
AI Cockpit の invocation は常に明示的な `--repo <path>` に bind します。
