---
author: AI Cockpit maintainers
title: "バージョニング"
description: "Runtime と Repository Protocol の version identity と migration boundary。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - versioning
---

# バージョニング

## 隣接 migration chain

Repository schema migration は、review 済みの隣接 edge からなる明示的な chain
です。Runtime は現在の schema から次の edge を解決し、未知の source、future
schema、または未 review の中間 version を飛ばす direct jump を拒否します。
approved な各 step は step identity、chain length、保持した historical evidence
digest、Runtime version/digest を含む Runtime-bound receipt を作成します。過去の
evidence、decision、knowledge、archive Work Item は byte 単位で保持され、migration
によって書き換えられません。

Runtime version、Repository Protocol version、repository schema version は独立した identity です。

```text
ai-cockpit --version
0.2.13

repository:
protocol_version = 1
repository_schema_version = 2
```

CLI version は executable package を示し、protocol version は repository storage contract
を示します。Runtime version、runtime digest、protocol version は `inspect`、`doctor`、MCP
`initialize`、verification evidence などの identity-bearing surface で一緒に提供されます。
`--version` は短い package-version command であり、完全な identity envelope を返す約束ではありません。

Runtime-only upgrade は compatibility が `COMPATIBLE` のとき repository の `.ai/` bytes を変更しません。
新しい verification と migration receipt には Runtime identity を記録しますが、Runtime は global な
active repository や Work Item state を持ちません。

現在の Repository Protocol は Protocol 1 のままで、attached repository の target schema は 2 です。
古い schema を黙って書き換えず、まず境界を確認します。

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

`COMPATIBLE` なら通常の lifecycle command を実行できます。`MIGRATION_REQUIRED` では inspect と
read-only plan だけを許可し、人間が明示 migration を review/approve するまで lifecycle、Agent、MCP、
verification は停止します。`INCOMPATIBLE` は fail-closed stop で、保存された schema を理解する Runtime
が必要です。Migration receipt は from/to schema、前後 digest、runtime version、runtime digest を bind
します。Work Item、evidence、decision、knowledge、archive history はこの migration で書き換えません。

Historical Work Item は decision boundary で使った Project Profile digest と protocol version を保持します。
Major migration は個別に review する Work Item とし、旧 evidence を保持します。
