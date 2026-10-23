# 给 Raydot 的精确四成员有限审阅提案（未执行、未批准）

当前实现已推送 sourceCommit `94e554ad836b19b4e027c3afda2159c4ace0fd18`。不是新确认、材料接受或正式验证证明。

当前 Contract `sha256:98aeef1116be2d93944042744ec8c9c8faba5043f43ae4267058290e3313e71e`，39路径/AC10/4 scenarios。实际新候选 SHA `e967d02a49806d0481272625bdc9813dc913a7aca1773aa734d4a095367b0a98`，版本1.0.1，独立路径 `/workspace/scratch/wi1090-additional-candidate-20261020/94e554ad836b19b4e027c3afda2159c4ace0fd18/ai-cockpit`；原候选与共享Runtime保留。

当前 preflight `sha256:5ea29c1b4286388c29b7d2c30ab98931a18b00d5b3c4739c14a2b6cc77c27e8d`，snapshot `sha256:880e5602b421577f2ae0819db92b3911ba03885a3f217ac7a08b24c1c9efa7b4`，状态yellow，实际记录于 `2026-10-20T03:23:20Z`。原confirm_review确实记录且保留；本次不能将其扩展为新材料风险许可。

当前材料请求 `sha256:6d419176c2e38bd790ee5b6c84dbc35a87e060b26e3fdfd88cab04c156423609`，manifest `sha256:76009df3df2d11067ad3e78261ba55899349dce0046866e0723da893f724525b`，material sourceSnapshot `sha256:25b60b47c1e677df3b9174dbb61689be6f37841c7febb1828cb8839e69433e09`，analysis implementation `sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83`。完整scanner/profile/policy/原生准入和磁盘字段见同目录JSON。Finding=0；Unknown code=1、成员=4；均为readable_committed_rust_syntax_unknown、reviewable=true。

- `crates/cockpit-cli/tests/verify.rs`
  - after blob: `sha256:2bc8aeed5b4f5ede41bb2eb7ed0167261368bab9afd3c9ee93cde0da730bbb84`
  - changed hunk: `sha256:27441ef578adf4d56db736e2b96f15d2ca5e52376dc9e963e668309855e7a249`
- `crates/cockpit-mcp/tests/rpc.rs`
  - after blob: `sha256:1b4ddbe9d24b98e24920808f913674ee01ba722f3ee429221d6da4847700738b`
  - changed hunk: `sha256:8c3a9e5bd225388cf57c28f78d49ac31fd7c5f4afddaba078cfeb7a821921ea7`
- `crates/cockpit-repository/src/material_review.rs`
  - after blob: `sha256:0ebba06d433ff1aa1a25fbc70d59d8735575958142438c89a813c6381d571a48`
  - changed hunk: `sha256:09435234cf7961bdc1248e876d9215cc57a5c61601c889f6946bcd43076c2e60`
- `crates/cockpit-repository/tests/lifecycle_order.rs`
  - after blob: `sha256:f971e470e9a4092f238dfa8fc8792ebf475c0fc9c707d27ac019f9ccf6a76d2d`
  - changed hunk: `sha256:9b6e660e749b97893fedf74ecfbf8387afd7b7e1766e67c9cf07b59e590c7f96`

三个测试路径是新发现的成员（固定继承模块之外）；测试源码本就在39路径内，材料风险授权没有因此自动扩大。此前真实批准只覆盖继承模块的0ebba/0943绑定。Contract原话："any new/foreign/unreviewable member, Finding or changed source binding requires a new decision." 原生profile仅限制原因类型，虽准入record_material_review_decision/run_verification，仍不能替代人已限定的精确成员边界。

建议的有限政策变更：仅将AC10中固定单成员审阅提案替换为以上四个精确blob/hunk成员的有限Agent审阅；保留禁止新/外来/不可审阅成员、Finding或身份变化的停止条件，self_declared不能声称扫描完整性。其他39路径、10项验收、4 scenarios、原六命令、所有历史保留和未授权merge/close/delete/release/Sentinel边界不变。authoritySource必须来自随后实际发生的授权，不预填或假设批准。原生修约会产生新的Contract/请求绑定，届时按真实完整绑定处理所需确认。

本提案建议：agent:Codex在新决定与原生准入实际允许后审阅当前精确四文件，记录真实review evidence，再录入typed decision；只接受这些可读语法Unknown的有限扫描盲区残余风险。它不是安全完整性、live授权、Provider或发布批准。

开发验证：仓库11/11与额外因果1/1、CLI1/1、MCP1/1。新增正向用例实际执行材料审阅→证据commit→finish/archive，无重跑；无材料阻断的因果用例证明旧恢复basis不能隐藏新失败，包括相同timestamp。正式新C六检查未启动。首次材料单成员assert失败后同shell下一条preflight仍启动，已保留真实exit0结果；没有材料接受、正式验证、finish/archive、PR或CI执行。

当前真正阻塞是精确材料许可，交给Raydot处理；不再向Ray重复询问常规技术问题。批准本提案也不等于已完成Agent审查，亦不预先确认未来Contract。
