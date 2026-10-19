# WI1090 最终源码材料绑定与首次 profile 草案

当前 profile=null、reviewEnabled=false；7 clean + 1 readable syntax unknown，Finding=0。本材料只更新最终源码绑定，不执行 opt-in、不生成同意文件。

- Source HEAD: `2785cfb2f90a3d4763f1b8d0ee7bf20585635138`
- Contract: `sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c`
- Request: `sha256:23e04c983732d47799e1ec444849fd89e0c92f3a5b13742ac636be30c8b6e3d8`
- Snapshot: `sha256:ffdcd5af8387e969dec46902088117057f82d986ef948d5e3d69c09776c52329`
- Manifest: `sha256:13d21e04a58c46874688c8998c67ea2582efe9be74f0070af3f37ad4ccc943ac`
- 唯一最小政策提案：初次 `/governanceProfile` set 一项；SHA-256 `ff93edd6797c0c6e973e1e3355024d84e96d4ca8d616604b37eec859785f323f`。`acceptResidualRisk=true` 仍仅拟议字段，不是已有风险同意。

| 材料 | blob | hunk | 评估 |
|---|---|---|---|
| crates/cockpit-repository/src/material_review.rs | sha256:0ebba06d433ff1aa1a25fbc70d59d8735575958142438c89a813c6381d571a48 | sha256:09435234cf7961bdc1248e876d9215cc57a5c61601c889f6946bcd43076c2e60 | unknown |
| crates/cockpit-repository/tests/material_review.rs | sha256:34c75a96e883a1f607c6097a48493b2b168507b9c709c7b56655cd9c1c617267 | sha256:5054c9c5a72d9966cfbadfd8374ba2bd5833d285d9d892fdb20d54a49bf44d90 | clean |
| docs/reference/reference-parity.ja.md | sha256:40486a8087e0e7afb5a820bac2c4b033282bd897463000ac1c898d11f87ab0c0 | sha256:4eb02a9316345f4220a3c6980072a97a4c8454c42f26b460870be74de84899e9 | clean |
| docs/reference/reference-parity.md | sha256:fa01b9b5d447ff4103e33acb6b6a9b240bb869eafe2ad18ec367bffaaec13b06 | sha256:ac43d3246dacfbcdc1673f661f70d0a673de9aeaef26dc9ae9b330d0f9581bcb | clean |
| docs/reference/reference-parity.zh-CN.md | sha256:2038ffe6e85cb06a3ec2ab8ca6fb3ce2155588300613bd38bfc7acb82babe380 | sha256:3053c29742dc0130a8b911d838d1b72fef269c5da68f73daebeaa1d79f5c9523 | clean |
| docs/work-items/WI-1090-authority-source-material-review-replacement.ja.md | sha256:a9bc68bc047f15b60510a8b4e4b7c308261f83c451cde5a21aeea2130a4ca24c | sha256:19b4a410853346d955b91d62921ab9633f2c7246d65a41a835e8166614576666 | clean |
| docs/work-items/WI-1090-authority-source-material-review-replacement.md | sha256:d4dfa5ee4bf2142dca780cd60613ddace011fd52bba6494d062ff301939fb1ad | sha256:07c5bab40b57c918d976fe414c3a513327bca2e5b68e4b02f3ee04f1f000f149 | clean |
| docs/work-items/WI-1090-authority-source-material-review-replacement.zh-CN.md | sha256:a332dc9c13d482403fd92697255274144e8a17542a117c647b5b1b99e31c02eb | sha256:a07dc24f029b3c5f59343ad07819c690b8d9294c6af748e715427cafa0f5d552 | clean |

全部8项 raw blob 与实际 u0 hunk 独立重算匹配原生计划。完整reviewable blob/u0 patch在外部保存，可用GitHub exact HEAD同文件审阅。

后续授权顺序：
- Raydot technical review of final source and minimum policy proposal; no actual risk acceptance inferred.
- After actual policy authorization only, natively amend; read resulting real Contract/preflight/snapshot/admission and obtain required bound Contract review if any.
- Refresh post-policy/post-documentation material plan; actual future request differs and is not predicted here.
- Obtain actual WI1090 residual-risk consent and reviewer decision bound to the real new material request. Native material record only if then admitted.
- Use resulting native verification freshness to determine any required checks; do not repeat unchanged checks preemptively.

M5.7 实现验收证明拒绝沿用旧批准/旧绑定；真实WI1090残余风险同意是独立未完成工作流。旧WI1089同意、技术审阅、合成fixture决定及 c9确认均不构成该同意。

未来 Contract/Request 必须取实际原生生成值；此处不预造绑定，不向当前材料批准推定未来材料批准。
