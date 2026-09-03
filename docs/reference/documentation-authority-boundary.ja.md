---
author: AI Cockpit maintainers
title: "Documentation authority boundary"
description: "人と Agent のための reader-first な documentation ownership。"
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# Documentation authority boundary

Canonical な Agent read set は repository-local です。`.ai/README.md`、`.ai/glossary.md`、`AGENTS.md` と、bind された repository の current な machine-readable `.ai` record を読みます。まず `docs/current/README.md`、次に adoption の `docs/getting-started/README.md`、詳細 command/semantic の `docs/reference/README.md` を利用します。language page は相互に link しますが、翻訳は presentation であり第二の policy ではありません。

Current/reference page は supported behavior を説明します。`docs/archive/**` の historical material は context に限られ、人が Work Item Contract に明示的に含めない限り current authority にはなりません。Source template の plan、Python script、Make target、generated report は比較 evidence であり、この Rust repository の指示ではありません。

Documentation check は frontmatter、link、locale counterpart、parity row、terminal evidence を検証しますが、draft を silent に promote したり governance decision を推論したりしません。Boundary や limitation を記載する場合は、対応する Runtime command、Contract field、evidence reference を明示し、object repository が source-specific installer、provider policy、wire format を継承すると claim しないでください。

Agent は action 前に Runtime state（`inspect`、`status`、`doctor`）を query し、current Work Item Contract を authority として扱い、handoff では visible な human Outcome を表示します。この route は全 attached object repository で共通ですが、`--repo` ごとの fact と decision は分離されたままです。
