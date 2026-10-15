## 新增依赖：敏感 Contract amendment 后的 `confirm_review` / `run_preflight` 准入循环

这是 #1041 的后续复现，请 AI Cockpit 组调查支持的恢复顺序。Sentinel Work Item `daily-radar-market-date-stdout-fix` 的产品提交已固定为 `f96211c8e3b2426735a0ddd2bc7becd455f5201b`；本地仍未执行新的 verification、finish、archive、PR 或合并。

### 精确状态

- 候选 Runtime：AI Cockpit 源码 `253dd007db00a01d3fe3a0e1e90cdc452eb7d5e8`，CLI `1.0.1`，二进制 SHA-256 `f6aed2b7f46b4a1b5d475d0d9c3f14900c2cf8021df19b9468fbb05b39906132`。
- 旧 Contract digest：`sha256:831d33cc75db638b258022b923ba7202c63c3db1087f61bd539720342aa3a8be`。Ray 已批准有限材料审查政策，随后通过候选 Runtime 的正式 typed `work-item amend --request` 入口应用（sequence 2，退出 0），没有手改 `.ai`。新 Contract digest：`sha256:e27b9841375dc64ef65a3c65b71b00c632025d48660c00bcf681b2f3d3d0c1e2`。
- 新配置只允许 `repository_material_inspection_unavailable` / `readable_committed_rust_syntax_unknown`，`assurance=self_declared`；Runtime 自动加入 `material-inspection-review` capability。材料计划现在 `reviewEnabled=true`、`findingCodes=[]`、`reviewDiagnostic=null`，request digest `sha256:192184b1e0a3d8abe8a8ac57d2fdbefe4b418e9005e8a407a004bdfda1a19d5a`。实际材料审查尚未完成，review receipt 仍为空；政策批准不是材料通过。
- Amendment 把既有 verification evidence 列为失效，并要求 `/governanceProfile` 的人类政策审阅。旧 `decisionEvidence.contractDigest` 仍绑定 `sha256:831d33cc…`；Summary 的 `preflightContractDigest` 也仍是旧值，`preflightState=not_run`。Ray 随后明确选择了针对新 Contract 的 `confirm_review`，但不能把这个选择写成不完整或旧绑定的收据。

### 最小阻塞复现

```text
<candidate> preflight --repo <Sentinel-test-worktree> --contract <active Contract>
exit code: 1
stdout: empty
stderr: current action admission rejected requested action "run_preflight": admission=NeedsHumanDecision, blockers=["governance_red"], safeActions=["record_governance_controls", "record_usage", "refresh_status", "resolve_blockers", "stop"], nextAction=Some("resolve_blockers"), admissionDigest=sha256:ef34e5eaacd8c4d28e0b737cecbade0c13e5a52b56334225fe0d6df89e577dea, issues=["lifecycle_gate_failed", "repository_material_inspection_unavailable", "user_visible_benefit_not_declared"]
```

`confirm_review` 的严格收据需要新 Contract digest、fresh `preflightDecisionDigest` 和对应 repository snapshot；但生成 fresh preflight 的 `run_preflight` 被当前 `NeedsHumanDecision` 准入排除。旧 digest/收据不能复用，也不能用政策批准冒充新 Contract 的 review。失败命令后 Contract 与 Summary 的文件哈希未变。

### 请求 AI Cockpit 明确依赖与验收

1. 给出此状态下**受支持且不降门禁**的顺序：如何先产生新 Contract 的 fresh preflight 绑定，再记录 Ray 已选择的 `confirm_review`；如为 Runtime 准入缺陷，请修复 `run_preflight` / human-decision 的循环。
2. 保持旧收据 fail-closed；新确认必须匹配新 Contract、fresh preflight decision 和 snapshot。材料 review 仍须另行对精确提交实际审查、留证，不能由政策批准或 `confirm_review` 代替。
3. 增加回归：checkpointed Work Item 已有旧 verification/decision evidence，敏感 amendment 使其失效后，Runtime 仍提供可完成的新 preflight → identity-bound human review 路径；foreign/stale/tampered 证据继续拒绝。

Sentinel 工作树、旧证据与日志保持原样；未尝试通过删收据、手写 Summary、复用旧 preflight digest 或制造材料审查结果解锁。

