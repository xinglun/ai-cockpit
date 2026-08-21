---
author: AI Cockpit maintainers
title: "Enterprise Governance Boundary"
description: "Enterprise adopter 向けの authority、policy、evidence、data、retention、audit の境界。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_governance_contracts
---

# Enterprise Governance Boundary

AI Cockpit は組織構造を指定しません。明示的な authority、十分な evidence、限定された scope、
見えている unknown、監査可能な decision を要求します。

## Authority と human decision

Authority evidence は `self_declared`、`repository_verified`、`provider_verified`、
`enterprise_verified` を区別します。各 record は actor、authority source、許可された operation、
policy reference、evidence reference を示します。Human decision は decision、actor、reason、
evidence/policy reference、時刻、任意の recovery condition を記録します。

Approval mode は policy が定義し、二人承認に固定しません。低リスクは承認なし、single authorized
human、multi-party approval、external provider approval を組織ごとに選べます。責任者が一人でも、
scope、fresh evidence、visible unknown、required check、decision receipt が明示されれば有効です。

## Policy precedence

Policy layer は organization → project → Work Item です。下位 layer は要求を追加するか上位 rule を
継承できますが、上位の approval strength を下げたり required evidence を削除したりできません。
弱化する overlay は fail closed で拒否します。

Runtime は任意の strict な `.ai/policy.json`（`schemaVersion: 1`、
`organization`/`project` slot）を読み取ります。Work Item は contract に
`layer: "work_item"` の `governancePolicy` を追加できます。有効な rule は
明示された contract `operation`（未指定なら決定論的な
`modify_source` または `production_destructive`）で選ばれ、自然言語で変更
されません。`preflight` は authority/evidence の不足を示し、verification は
権限不足の operation を実行せず、`finish`、`archive`、`close` は effective
decision が green の場合だけ進みます。Policy 対象の close は structured
decision と `policyRefs` の policy ID binding が必要です。Multi-party と
external-provider mode は外部 approval receipt が import されるまで
fail-closed です。

## Delegated evidence と audit boundary

External provider は自分の proof を生成する責任を持ちます。Delegated evidence model は provider、
subject、origin、assurance、収集時刻、digest、validity、raw evidence reference を bind します。
AI Cockpit は require、validate、display、archive できますが、provider signature、branch protection、
SBOM、provenance、enterprise approval を生成したとは主張しません。

`ai-cockpit evidence import --repo <repo> --work-item <id> --metadata
<metadata.json> --raw <provider-output>` で provider metadata を raw bytes の
digest に bind します。Raw reference は `.ai/evidence/external/` 配下に限定されます。
同一 bytes の再 import は idempotent ですが、receipt の衝突、path escape、symlink、
unknown field、repository/Work Item mismatch は fail closed です。`ai-cockpit evidence
list` と repository-bound MCP の `delegated_evidence_list` は再検証済み receipt だけを
表示します。Expired、revoked、unknown receipt は監査用に残せますが、
`delegated:<provider>` evidence requirement は満たしません。

Audit event は安定した event ID、repository/Work Item identity、Runtime identity、時刻、digest、
evidence reference を持ちます。Local Git と `.ai/` を独立した immutable enterprise audit log とは
主張しません。高い assurance が必要なら SIEM、WORM、S3 Object Lock、enterprise audit system、
external ledger へ export します。

## Sensitive evidence と retention

Evidence classification は `public`、`internal`、`confidential`、`restricted`、
`secret_prohibited`、persistence は `full_capture`、`redacted_capture`、`digest_only`、
`no_persistence` です。`secret_prohibited` は full/redacted capture を許可しません。Retention
metadata は expiry と deterministic な disposal action を記録します。Retention policy は purge plan
を要求できますが、AI Cockpit は歴史 evidence を黙って削除せず、local archive が法的 retention を
満たすとも主張しません。

これらは enterprise compliance を支援する technical control であり、ISO 27001、SOC 2、その他の
組織認証ではありません。
