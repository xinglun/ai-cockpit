# WI-1087 P1 恢复交接

源码修复及新证据已经分阶段保存；P1 的完整正式验证仍未就绪，整个 WI 保持开放。

- 同一 AI Cockpit Rust Dev 环境；未创建第三环境。Runtime 1.0.1 从声明的 a12ee2ff4413ac261560339c9ed7af59dde0f701 源码实建，实测 SHA-256 为 0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4。
- Contract 原生 digest：sha256:b6a6be5bb53bb857053b1167e9960ea1382ff9eccf98bc7ca37893e26d610e86。原始 9ca 人类 preflight review 字节不变；没有重放 controls/start/checkpoint 或制造新真人决定。
- 授权四类起止/cap 字段检查完整原始小数尾数；SystemClock 夹读并拒绝跨毫秒桶间隙，保留原 UTC；remaining_millis 文档与测试明确 fresh sample → observe 后作判断。
- 新 focused：core 19/19、protocol 12/12、authority 17/17（152 filtered）；新 repository library 单包诊断 169/169（0 filtered）；三包 all-targets Clippy -D warnings 通过。这些不是整包及整轮 workspace 通过声明。
- 原生 verify 两次均在 890 秒退出 124；最后进度均为 11/13，repository 包无终态，verification 包未启动。v03 只读进程观察显示进入 legacy_closeout 集成测试。两次超时日志完整保留；没有完整原生 workspace 收据。
- 当前原生 status：checkpointed、admission allowed、blocking=false、humanDecisionRequired=false、verification=not_ready；unknowns 两项为 user_visible_benefit_not_declared 与 verification_evidence_missing。validate 另有 intent_alignment_missing warning。
- 原生中文 Outcome 已实际返回黄色。其“人工决定：未记录”原文保留；历史 preflight review 与当前完工决定不混用。没有执行 Outcome 泛化建议中的 finish。
- Draft PR： https://github.com/xinglun/ai-cockpit/pull/1046 。87ac1c86 对应 CI 39005781978 最新观察仍 in_progress；route 与 v1-behavioral-oracle 成功，quality 与 windows-runtime 未终结。后续 push 会产生新的 HEAD/CI，必须重新读取，不可把此观察当新 HEAD 成功。
- 旧断连终态和未推送日志继续未知；本次不补造。实际当前两次超时终态已知，不与旧未知混同。当前目标进程组仅剩 zombie cargo 条目、无活跃成员；没有引入 supervisor。
- 最近磁盘：workspace 27 GiB 可用，tmp 8.8 GiB 可用。

最小下一步：先读回最新远端 HEAD/CI；为完整 workspace 检查确定受支持、每命令不超过 900 秒的原生编排路线。没有批准改变时限或弱化检查，本次停止重复整轮。若路线需要新的真人决定，应给出原生完整绑定。P2–P4、真实 grant、Sentinel、合并、finish/archive/close 和删除均不在本次后续范围。

本文件为 agent:Codex 的事实交接摘要，不是 Runtime receipt、真人批准或全功能验收。代码边界修复对未来授权执行的实际收益尚未通过 live 系统验证。
