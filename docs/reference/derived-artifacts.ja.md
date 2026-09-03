---
author: AI Cockpit maintainers
title: "Derived artifact と authority boundary"
description: "Rust Runtime の projection を観測可能なまま governance authority にしない境界。"
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# Derived artifact と authority boundary

AI Cockpit は repository fact と、その fact から導出した view を分離します。Contract、repository snapshot、verification receipt、decision、archive manifest は typed identity と digest binding が検証された場合だけ authority です。Status、summary、Outcome handoff、knowledge index は人と Agent 向けの derived projection であり、変更を authorize したり source record を置き換えたりしません。

Reference template には generated fact と artifact input を検証する Python registry があります。Rust Runtime は explicit input、source reference、deterministic derivation、fail-closed identity check という portable rule を保持しますが、その registry や JSON wire shape は copy しません。Repository-local Knowledge も read/derived view であり、Contract、Evidence、human Decision の代替ではありません。

Audit では source record を先に読み、その後 projection を確認します。

1. `ai-cockpit inspect --repo <repo>` で snapshot と changed paths を確定します。
2. `ai-cockpit status --repo <repo>` で lifecycle fact と readiness を確認します。
3. `ai-cockpit work-item outcome --repo <repo> --id <id>` で human handoff を表示します。これは新しい decision ではありません。

Projection と source が一致しない場合、Runtime は binding または freshness problem を報告して停止します。Agent は generated status、Outcome、knowledge、evidence、archive を手編集せず、人の明示的 authority で owning Contract または Runtime operation を更新します。

Attached object repository は shared binary と明示的な `--repo` context を通じて同じ boundary を継承しますが、source Python registry や source-specific generated-file policy は継承しません。
