# WI-1087 P0 规划接续与原收据持久化

## 范围与真实授权

本文件由 agent:Codex 编写，不是 Runtime 决定或真人批准。Raydot 主线程 01a11856-d6ba-72e0-9aab-25a34cd2f018 明确指示执行已审阅 P0，在用户委托下登记当前上下文、正常同步 main、原生两收据窄 scope amendment、规划提交/分支推送/Draft PR/CI。该决定是普通 Agent 技术计划审阅，不是 human confirm_review；不授予产品实施、Runtime 激活、PR 合并、删除、凭据创建、扩权或直接 main 推送。

## 实际上下文与原生步骤

- 原 source：eb942a0819a40173842d22e90841c8b95c02870f；main：1174621c397feb2849812f6ccee4219eb9b6096b。
- 独立工作区：/workspace/ai-cockpit-wi1087-contract-authority-scope-and-expiry；同一分支 codex/wi1087-contract-authority-scope-and-expiry。
- 正常 merge：cc3ff71c5bb8dbdaea583134bdb483c167e225cd；两个原 HEAD 均保留为父提交；tree=281746c50e4e38884685a980f56eaf46b3166eeb。没有 rebase、reset 或强推；main 三语 parity 字节及 v1/v2/v3 原件保留。
- 本环境 Runtime 1.0.1 的 SHA256 为 0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4，固定构建来源 a12ee2ff4413ac261560339c9ed7af59dde0f701；不声称旧安装字节连续。
- 首次 registration 因初始 PATH 无法解析 Cargo 失败，原输出保留；仅为后续进程设置已存在的 cargo/rustup 路径后成功，版本 rustc/cargo 1.98.1，未修改全局配置、安装工具或运行构建。
- Runtime 实际登记 generation1，正常合并后 generation2，原生 Contract amendment 后 generation3。每次使用真实 head/Contract/runtime；实际环境基线 sha256:67a5f6afcf18be42280c1dee66bfe0e26fb8e07911fc9980733b8ee32a0042d6。登记是任务上下文/环境观察，不是实施授权、已发布 Outcome 或并行 lease。
- 相应三次原生 drift check 均成功返回 pending=false；仅相对本次真实基线，不证明旧环境→新环境连续性。身份更新生成的真实 auto-impact 事件保留；目前没有受影响消费者/blocker，不伪造 recovery。

## 原生窄 amendment 与真实门禁

原生 sequence=2；requestDigest=sha256:53fbc5a9049e978ef694a694ece45ecd9845c68c70e366b16b6d75c6944f2e28；journalDigest=sha256:307d55c9a476073bcaa71194f5ed2b01f543e0e9e7a2c05805d91f19aafec648。

Contract 从 sha256:a2bc3623b4953e76702b9c0618aaf739520d6e2c41345f259e432f5273641cc7 变为 sha256:7f827f8bf5b6eaacd599045f243f74ee3a89998bde781fc5aae65af83f877eee。操作只有：scope 新增两份精确既有 receipt 路径，以及 outOfScope 原字节新增一次的窄例外。Native 生成 prepared/committed 记录，未手改 Contract/Summary/历史。旧 base=681df30e2fe8c869b6a41498822818c017dc4fb6 保持。

Native receipt 的 policyReviewRequired=true、nextAction=record_governance_controls 如实保留。当前 mode=planning、state=not_ready、authority=missing、checkpointCount=0；没有 start、preflight、confirm_review、正式 verify 或产品实施。当前 needs_human_decision/verification missing、材料 Unknown 及 planned 文档的验证投影诊断不被声明解决。尚未生成新的真人 preflightDecisionDigest，也不把 amendment requestDigest 或 admissionDigest 当成人审请求。P0提交仅供规划材料审阅，不声称实际实施已获准。

## 两份既有 cleanup 收据

两原件分别为1020/1038字节，原字节直接复制到相同 Native 路径，见 receipts.json。WI1085 rawSHA=d85bcc5d38ed159e1c92a409d0a0b8feeb36bb16c41826d0e11531e5c565a9b5；WI1086 rawSHA=7304e582f78f5a49ec751f01f5497a3a310a66c5621fa3353813869c961a6a8d。

本次没有重新执行 cleanup 或产生新的清理观察；close/archive/旧收据字节不改。主工作区原件及/tmp副本保留，直至远端提交原字节读回成功。共享 stash 与/tmp/wi1085-result-20261016.git 不在清理范围。

## 设计与后续实施边界

继续既有 v1/v2/v3：一次明确有限目标窗口可覆盖被明确授权的连续/并行WI、阶段及正常修复/重试/恢复，刷新动作准入不等于反复逐步骤续批；旧目标不得用于新目标。900秒是单命令上限，不是目标授权期限。未配置实际 grant 或 expiry 功能；grantId/subjectRef/notBefore/expiresAt 仍是未来设计字段，不冒充当前 CLI 参数或已启用能力。

P1仍仅五文件候选（protocol lib、core clock/lib、repository authority/lib）；P2来源覆盖/版本/准入，P3真实受管停止和metadata-only延期，P4 CLI/MCP及独立迁移。31项基础验收及R04的18变体均not_run，没有将其他模块或旧测试当作本项通过。

旧规划先按正常PR集成main，保持祖先；同一工作分支再正常快进到实际main，才可能在真实实施授权与新鲜原生准入下start。本P0不绕过该基线，不伪造前驱或改变现有权限。PR/CI的实际source与tested revision及终态需另行读回；本文件不预告CI通过，也不以CI替代Native正式验收或真人决定。
