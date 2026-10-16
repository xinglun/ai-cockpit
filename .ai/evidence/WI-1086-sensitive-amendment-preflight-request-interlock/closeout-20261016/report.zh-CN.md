# WI-1086 原生归档关闭与待审清理清单

Outcome: 🟡 产品已合并；专用 worktree 中原生归档关闭成功；治理记录整合与清理待后续处理。

## 执行授权与交付

当前用户明确转达 Ray 批准 PR1042 转 Ready、合并完整 source head eac3e5a665a98efd1436a7100d1e40feaf0003c2，并按原生流程归档关闭 WI1086；Raydot 技术审阅通过；清理先报精确清单。执行者是 Codex，未冒充 Ray 操作 CLI 或提交 GitHub review。

执行前实际核对指定 head、PR base681df30e、四个CI jobs成功、native verified/fresh 与 archive_when_reviewed 准入。通过GitHub工具转Ready后以expected_head_sha保护merge操作。实际merge commit为a05a572d3c03cb45ec602eab3db587a19f77acf3，非CI合成提交。远端分支未自动删除。

primary /workspace/ai-cockpit 已同步实际 origin/main；同一WI分支的专用linked worktree为 /workspace/ai-cockpit-wi1086-closeout。创建后实际Runtime查询仍verified/fresh、无blocker，未声称靠路径推断环境连续性，未重跑13节点。

## 原生 archive / close

原生archive成功，生成immutable archive，提交为de6f2da9bf9a5f3f2f3f0d71b1928e0e6ab38958，以满足close对archive bytes at HEAD的要求。archive manifest原字节SHA256为281a412784d1b64b40bd22c425263454f9beb15b67982aca2de95fa866bf6885。

随后原生safeActions明确包含close_after_review。第一次CLI结构化close输入缺少必需decidedAt，在执行前被parser拒绝；原始空stdout与stderr保留，没有伪称它写入决定。补入本次真实记录时间后成功。structuredDecision将当前用户作为授权转达来源，reason明确decidedAt是此次记录授权决定的实际时间，非独立获知的Ray原始审批时间。未复用旧confirm_review作为close批准。

关闭后的Runtime实测closed、verified/fresh、blockers=[]、humanDecisionRequired=false。批准decision=approved。普通路径的resourceContext为空，没有伪造provider finalization收据。

## 关闭后受控文档投影

真实治理checker先报三条stale_prearchive_parity_registration；保留原始失败。仓库既有promote_closed_work_item.py先check，确认需要六个受控文档修改，再执行已支持的promotion；没有手改Contract/Summary/receipt或放宽validator。helper实际接受no-resource close，terminalFinalization=null。

结果：三语WI页面与三语parity行共六文件受控更新。--check-all state=current，checked=51；随后parity_status_check通过914个work items。原native验证receipt未改变，原始13/13、0复用、0超时证据保留。提交后再查询Runtime以确认material projection。

## 清理候选（未执行）

本地分支：refs/heads/codex/wi1086-sensitive-amendment-preflight-request-interlock。
远端分支：xinglun/ai-cockpit 同名 refs/heads/codex/wi1086-sensitive-amendment-preflight-request-interlock。
专用worktree：/workspace/ai-cockpit-wi1086-closeout。
worktree管理身份：/workspace/ai-cockpit/.git/worktrees/ai-cockpit-wi1086-closeout，worktreeId sha256:961aff6be22dce785797a9e015bf04bfed5762bd13c3bc6f90c2957af241644e。该Gitdir只作为身份核验，不另行raw删除。
close时捕获head为de6f2da9bf9a5f3f2f3f0d71b1928e0e6ab38958；关闭记录提交将推进分支，实际最终HEAD必须再次读取并在删除前核验，不能把close绑定head误当成更新后的分支head。

保留primary /workspace/ai-cockpit及其.git、宿主/workspace/.git、Runtime、缓存、其他WI与旧环境。当前未调用ordinary-cleanup、未删除任何分支或worktree。先把本次治理记录整合到默认分支并获得清理授权，再执行精确清理和原生ordinary-cleanup观测记录。产品PR1042的批准只用于原先eac3e5a6 source merge，不静默合并额外治理提交。

## 保留未知项与交付边界

intent_alignment_missing、user_visible_benefit_not_declared、usage no_usage_receipts仍按Runtime原样保留；没有虚构用户收益、风险完成检查、用量或审批。新生效closeout治理提交应由Raydot审阅后决定默认分支整合。primary当前main已包含产品merge，但尚不包含本次后续原生归档关闭记录；不得把专用worktree的closed状态冒称主线记录已同步。
