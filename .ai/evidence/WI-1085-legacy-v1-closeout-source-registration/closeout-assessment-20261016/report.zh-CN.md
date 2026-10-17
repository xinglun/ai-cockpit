# WI-1085 正式结果与收尾交接

Outcome: 🟡 正式验证实际通过；完整验收及集成尚未完成。

## 正式终态（已于 ecc881fd 推送）

- 测试源码 HEAD `5c0212c6083abbdc6843c2e2647a6de283232e3d`，实现提交 `284e31bed828c7cd04bb41261f5059ad8f1d6e69`。13 个节点均实际 spawned、passed，0 失败/复用/超时；842538 ms，最大并发 1。focused 1+3 项通过及 13 包 compile-only 准备分别记录，不算正式替代。
- attempt `sha256:e749e39f2d3bce872b6bb6e20a912ea3c72d549e38e31f3d2817c78eecf3aec2`；canonical receiptDigest `sha256:92989aac97929788723d885157bd1767ae39e7b31a2dd93bf4403409d81eecee`。
- Contract `sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1`；repositorySnapshot `sha256:bddd1a809e3b04e357272e17cd1771cb80fb312f16301bd33f688ea4522a535e`；Runtime `sha256:0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4`。
- Raydot 对 ce6cf9c9… 的实际 accept_permitted_unknowns 已原生记录为 b8da3c27…，仅原两份允许 syntax Unknown，assurance=self_declared；真实 Ray confirm_review cf2… 保留原绑定。没有伪造新审阅或复用旧绑定。
- 正式前后进程证据、26 份短路径 stream 原字节/哈希、截断标记、原始 Runtime attempt/canonical 位于相邻 `verify-imports-20261016/`。repository stdout 原生截断仍公开，未补写尾部。旧失败原样保留；先前 Bus error 未重现，不作因果证明。

## 分别评估场景、意图和收益

- `scenarios.json` 列出全部 11 项，以 53 条实际 captured success line + 未过滤包执行记录映射，源码位置和 SHA256 一并记录；不是伪造单 case receipt。10 项 verified。
- `filesystem_boundaries` 仍 unverified：Linux bounded reads/duplicate keys/path alias/parent replacement/symlink/hardlink/victim/FIFO 等有实际证据；A6 和计划明确包括 reparse，当前无 Windows 执行。测试 HEAD 已有 hosted check-runs 为 0。未用 Linux 通过或静态源码假称跨平台完成，也未擅自豁免要求。
- `criteria.json` 分列 17 项支持材料，A6 明确 incomplete；映射不是最终人类验收。`intent.json` 说明核心维护者目标有 fixtures 支持，因以上完整性缺口保持 unresolved。
- `benefit.json` 独立保留 unknown。降低误信 closed/错误释放 scope 的风险是推断，未观察真实用户效益，未实际登记 WI-1004；全部本地锚点消失/回滚不可识别的边界保留。
- Runtime 已允许并成功记录 `controls-input.json`；未手改 Summary。当前 validate：acceptanceEvidence/evidenceClasses verified，scenarioCoverage unknown，唯一细项 `required_scenario_unverified:filesystem_boundaries`，intentAlignment unresolved，findings=[]。
- 当前 status：verification verified/fresh，blockers=[]，humanDecisionRequired=false；四个 unknown 仍为 governance_controls_unknown、intent_alignment_insufficient、scenario_coverage_insufficient、user_visible_benefit_not_declared。没有新增真人门禁，不重复询问 Ray。
- 原生 Outcome 原文保存在 `outcome.zh-CN.txt`；它标题绿色仅指已声明验证通过，正文同时报告场景/意图不足。对外交付仍为本报告黄色。Runtime Outcome 的“人工决定未记录”不抹去已存在的 preflight 人审；不宣称 final acceptance 已有真人决定。Host 是否展示该原生输出未知。

## main 集成差异及精确最小路径（尚未执行）

- 远端分支经 ls-remote 确认 ecc881fd，main 为 `9e370e1c2c2e2dceda79de62ca4e849ae313624c`。共同基线为 `681df30e2fe8c869b6a41498822818c017dc4fb6`，main 不是当前 HEAD 的祖先。
- `git merge-tree --write-tree --messages --name-only` 只读诊断返回 1；实际 HEAD/index 未变。synthetic tree 不代表可交付或已验证合并。
- 唯一内容冲突清单：`docs/reference/reference-parity.md`、`docs/reference/reference-parity.zh-CN.md`、`docs/reference/reference-parity.ja.md`。源码 lib/status_projection 自动合并，但仍不受当前正式收据覆盖。
- main 有 933 个 delta paths，其中 928 不在此 Contract scope，多数是已完成 WI-1086 治理/证据；实际非 .ai 路径列表在 `main-integration-facts.json`。不能将 upstream 已合入事实冒充本轮 reviewed/tested source。
- 最小建议：由 Raydot 先审阅这份精确同步差异，按原生现有 baseline/admission 流程继续同分支；三份 parity 文档合并时保留 WI-1085 和 WI-1086 两侧真实行，不引入新功能、不改断言或手改 Contract base。源码或当前绑定变化后必须重新核对 admission/material 及实际验证，e749 保留为历史，不覆盖或强推。
- Windows 及集成验证需在获准的 PR/CI 路径补齐；本轮不创建新 PR、不转 Ready、merge、archive、close 或扩大 cleanup。普通技术评审交 Raydot 判断，不用该建议制造新的人类审批。

## 回收、保护与下一步

正式根进程已回收，后续只读终态 census 无 cargo/rustc；精确空间见 `final-processes.json`，正式结束 6446010368 B；后续终态 census 可用 6443294720 B、全局僵尸 1432（正式窗口前后均 1431，窗口外增加不归因于本次测试），无未解决读取错误。仅此前获批旧 WI-1086 debug 缓存已删，无其他清理。旧 release Runtime、失败证据、主 checkout 独有 cleanup receipt（SHA256 7304e582…）仍保留。本轮没有重跑测试或改产品/断言。

下一步由 Raydot 接收正式成功及本轮治理评估，处理精确 upstream 同步与 Windows 覆盖缺口，再按真实原生准入推进 WI-1085 收尾。后续顺序保持：1085 收尾 → Raydot 在对应 Sentinel Issue 回复“请复核” → 1087 权限到期 → 历史 branch/worktree 清理 → 其他非阻塞改善 → Task9；本执行方不代发 Raydot 的 Issue 回复。
