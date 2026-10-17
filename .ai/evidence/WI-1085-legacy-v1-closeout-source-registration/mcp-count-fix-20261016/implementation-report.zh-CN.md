# WI-1085 两行计数修正：实现提交前事实

Ray 在父线程实际回复 confirm_review（Sentinel_66886c257f5c8191968f2b12ceb7d8de，2026-10-16 12:49 UTC，时间精度为分钟）。核对1947ce33载体和全部当前原生绑定一致后，Codex以真实录入者身份通过 native controls 记录；原生生成新的 cf2cf5b7… preflight-review receipt，未改旧人审或材料收据。recordedAt为实际录入时间，不倒填为回复时间。

绑定仍为 repositoryId ee02a04c…、WI-1085-legacy-v1-closeout-source-registration、Contract7fea3f7d…、review snapshot6bcd443d…、preflightDecision7b2ccbca…。记录后 native status humanDecisionRequired=false、admissionState=allowed、diagnostics=[]。

落实已经Raydot技术批准的精确变化：material_review.rs:88 与 usage.rs:25，各将 tools.len() 期望31改为33；两文件逐字节等于旧版本仅替换该一行，其他断言不变。TOOL_NAMES为33，其中新增plan/record两工具；现有rpc测试也已断言33。没有产品源码、Contract、validator、验收规则或其他测试修改。

修正后的 preflight 仍确认 human_decision_recorded、humanDecisionRequired=false；repositorySnapshot更新为8a858ff1…。它报告material_review_projection_unavailable。只读material-review plan明确解释：两份非.ai源码有未提交修改。因此按当前任务已授权的“提交修正后刷新最终材料”步骤提交，解决该投影前置；不忽略此阻塞直接验证。该原始只读错误和返回码1均保留。

历史native失败attempt a1f426…是真实红灯，material_review实际为33≠31；usage未执行到，仅静态确认同一旧计数。不伪造新的红灯或绿色结果；本轮尚未运行任何测试。提交后必须重新生成实际committed source材料请求，交Raydot作新的身份绑定审阅，再按准入进行正式验证。原94bdbf…材料决定和中间81878b…请求均不充当最终材料批准。
