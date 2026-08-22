---
author: AI Cockpit maintainers
title: "WI-110 — Evidence assurance と historical projection"
description: "Strict verification evidence、current Runtime binding、legacy projection。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-110-evidence-assurance
---

# WI-110 — Evidence assurance と historical projection

## Intent と goal

verification trust boundary を明確にします。保存される v2 evidence envelope と captured
receipt は typed、identity-bound、fail closed でなければなりません。current CLI lifecycle は
インストール済み Runtime が生成した evidence だけを受け入れます。pre-v2 の historical bytes
は immutable のまま保持し、現在の失敗を偽造せず historical input として表示します。

## Scope

- strict `VerificationEvidenceV2` envelope と nested `VerificationReceipt` validation;
- Work Item、repository、snapshot、Runtime identity の binding;
- Runtime-bound CLI/MCP verify、finish、archive、close、Outcome path;
- unknown field、nested identity 欠落、malformed、foreign Runtime、legacy evidence の regression;
- English、简体中文、日本語の documentation。

## Invariants

unknown な envelope/captured receipt field、nested identity の欠落、invalid digest、foreign
Runtime identity は green Outcome を作れず、Runtime-bound lifecycle を通過できません。
`digest_only` retention には captured receipt がありません。pre-v2 record（
`evidenceSchemaVersion` がないもの）は read-only historical input で、黄色の
`legacy_evidence_historical` として projection します。rewrite、green への昇格、現在の赤い失敗として
の表示はしません。v2 record の identity 欠落は赤色のままです。

明示的な `RuntimeContext` を持たない compatibility Rust API は Runtime identity を自分で管理する
embedder のために残します。installed CLI と repository-bound MCP は常に Runtime-bound API を使います。

## Verification

focused evidence/lifecycle test は strict envelope/nested receipt tampering、foreign Runtime 拒否、
CLI foreign-runtime 拒否、immutable legacy projection を対象にします。merge 前に workspace format、
Clippy、full test を通過させます。

## Boundary

provider attestation、外部 immutable audit storage、historical bytes migration はこの Work Item の範囲外で、
別の enterprise assurance work とします。
