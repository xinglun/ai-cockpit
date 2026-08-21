---
author: AI Cockpit maintainers
title: "Runtime 拓扑"
description: "Runtime 组件和 repository 持有的存储路径。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - runtime_topology
---

# Runtime 拓扑

```text
Human / Agent / CI
        │
    CLI / MCP adapter
        │
        ▼
    cockpit-core
        │
 ┌──────┼────────┬─────────┐
 ▼      ▼        ▼         ▼
Git  Repository Evidence Verification Knowledge
        │
        ▼
  对象 repository 的 .ai/
```

`cockpit-core` 是纯且确定性的。Adapter 把外部请求转换为 core input，再把决定
转换为 CLI/MCP response。Repository 访问通过显式 port 提供；core 不能遍历
filesystem，也不能调用 Git。

对象 repository 只保存 facts、decisions、evidence 和生成的 knowledge projection，
不保存 Rust source、Python runtime、helper scripts 或 runtime schema copies。
