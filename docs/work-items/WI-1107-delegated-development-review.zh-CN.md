---
author: AI Cockpit maintainers
title: "WI-1107 — 完整起项与显式受托开发审阅"
description: "Complete initial scenario plans and finite explicit agent technical review; production activation excluded."
audience:
  - maintainer
  - reviewer
workItemId: WI-1107-delegated-development-review
status: in_progress
authority: work-item-record
lastVerifiedBy: WI-1107-delegated-development-review
---

# WI-1107 — 完整起项与显式受托开发审阅

CLI `start --scenario-coverage <json-file>` 接收既有 typed
`ScenarioCoverageEntry` 数组；MCP `work_item_start.scenarioCoverage` 使用同一类型及仓库校验。
初始场景必须未验证、证据为空，在首次 preflight 前保存。无效输入不创建生命周期记录；省略选项保持旧形状。

CLI `start --development-review <json-file>` 或 MCP `developmentReview` 显式选择工作项策略，
保存于 `governanceProfile.developmentReview`。默认仍严格；包含该选择的精确基线仍须正式原生所有者审阅。
actor、来源字符串或任意批准文件不能单独建立授权。

```json
{"schemaVersion":1,"reviewerActor":"agent:reviewer","authoritySource":"user:explicit-bounded-delegation","allowedChanges":["append_scenarios","fill_verification_plan","append_verification"],"preapprovedCommands":["cargo test --locked"]}
```

typed amendment 的可选 `developmentReview` 复用 AuthorityEvidence：明确 agent／来源、
`assurance=self_declared`、唯一操作 `review_development_amendment`。
只允许三类变化：追加必需、未验证、空证据且使用精确已声明／预批准命令的场景；
用该类命令补齐既有空 verificationPlan；追加有限清单中的精确预批准命令。既有顺序及检查保留。
scope、权限、风险、策略、验收标准及任意命令改写仍须真人或拒绝。

Runtime 在原生命周期锁内，将独立 `delegatedReview` 绑定到原生 prepared／committed 修订链：
所有者基线及回执、策略、repo／WI、请求、修订前后 Contract、源码快照和 Runtime。
缺失、改名、篡改、跨项或不完整记录被拒。旧所有者回执不变；代理记录不是 `decisionEvidence`，
不能解除 close、Finding、失败验证、更严格的真人／provider 策略或宿主／GitHub 审批。

这是本地来源记录，不是外部身份认证，也不承诺抵抗改写全部本地可写记录的攻击者。
本项不替换共享 Runtime、不启用生产配置。生产使用须另行显式选择并取得新所有者审阅，代理技术判断可能出错。
跨 WI 授权期限／撤销与外部 B 不在本片；隔离测试减少请求次数不等于已测量实际收益。

范围为 9 份生产源码、4 份测试、12 份三语文档及本 WI 原生记录。治理使用固定正式 v1.0.1 b2，而非开发中的能力。12 项场景及九条命令必须通过真实验证和独立全分支评审；RED／失败原样保留。公开发布、合并和真实 close 仍须对应授权。本片不等于完整 WI-1087 已完成。
