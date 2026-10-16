# WI-1085 精确恢复与原生 scope amendment

Outcome: 🟡 空间恢复和安全对齐已完成；原生 amendment 已记录，当前停在新的真人 preflight 门禁。两份测试尚未修改、未重新执行测试；不是正式验收。

## 已完成

仅删除已批准的旧 WI-1086 debug 编译目录，净可用空间从 0 增至 11,884,752,896 字节；保护对象哈希/元数据一致。原本地25份未跟踪证据在具名 stash 与恢复后的远端树逐字节相同，已正常快进至 32353165；无 reset、强推或额外清理。详见 ../host-storage-recovery-20261016/report.zh-CN.md。

真实父线程转述 Raydot 技术批准的原 request raw SHA256=04eb809ea57c5d4392717ca88020fe754d11139aa0dd33979f1ac3cd07d07a57，未改字节直接交 native amend。原请求文件名/理由中保留了 DRAFT 字样，它是历史提交时的原始文本；本次实际应用事实以 native-amend.json、技术批准转述和 sequence5 prepared/committed journal 为准，不把它说成从未应用。

Runtime 在 2026-10-16T12:38:06Z（客户端时区 21:38:06 JST）实际记录 sequence5，journalDigest=sha256:cf98ff230bb7c31371ef7b090bb450099279da87d86d0e978a6d11979d02c54f，native canonical requestDigest=sha256:55f7cf6601c09508c391a927299ee7f622b9fdbd627524dc6e3ff4650a751db4。raw 文件 SHA 与 native canonical digest 是不同概念。新 Contract sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1；旧 Contract sha256:eae15cbbdbc36606635db999976611d3c036260296d2b5b24b9afa183ae03bd5。

逐值核对 Contract，只增加两条 scope 与两条 concurrencyBoundary.implementationPaths；其他值完全一致。两份 MCP 测试、产品、现有验收/断言均未改变。

## 当前真实阻塞

Native preflight 返回 needs_human_confirmation，humanDecisionRequired=true，缺少当前新 Contract 绑定的真人确认。实际需要的绑定完整见 human-review-request.zh-CN.md 与 current-required-bindings.json；推荐 confirm_review。当前 safeActions 不含 run_verification，因此没有尝试验证来制造零进程拒绝记录。Raydot 的技术批准不充当真人确认。

Native status 同时确认旧材料收据 stale，reviewReceiptDigest=null。只读刷新得到 requestDigest=sha256:81878ba626fe354bfbbe24f1fd573c57485e41d73942a7173b5a1e3b9894be63，旧为 sha256:94bdbf788b5bc75f54766d9cff37830f166b57cf6aa5ef629df84525bc4728d1。变化字段仅 contractDigest、requestDigest、reviewedSourceHead；sourceSnapshot a839198a…、manifest7ca05a11…、profile/analysis/policy 均保持。Entries仍25、Finding0，保留原两份 legacy_closeout_source.rs readable_committed_rust_syntax_unknown。旧 e093474e… 决定原字节保留而不复用。

这个新材料请求是**修改测试前**的实际请求，提供 Raydot 审阅当前 Contract 绑定；两行测试修改获准实施并提交后，还必须再次刷新实际 source request，届时记录真实 Raydot 材料决定，不能套用94bdbf…或当前中间绑定。没有伪造任何新材料批准。

## 下一次验证空间规划

当前隔离 target 已有 19,205,693,440 字节，仍保留。只读盘点 repository 顶层测试51个，现有匹配测试可执行文件最大151,399,848字节。保守假设全部51项再产生一份最大体积变体，预算7,721,392,248字节，加链接峰值512MiB及其他 package/日志/重编768MiB，估算增量合计 9,063,569,528 字节（8.44GiB）；另保留2GiB可用空间，启动阈值 11,211,053,176 字节（10.44GiB）。盘点时可用 11,881,852,928 字节，满足该估算阈值，但这不是构建大小的数学保证。

正式执行前必须重新读容量并核对真实准入；未满足阈值则不启动，运行中持续监测资源，出现观测失败或增长威胁保留空间则停止并重评，不能再等到磁盘满。没有进一步清理授权。保持单 worker、单 build job、单测试线程、同一隔离 target、现有 Tini/observer runner、890+10秒外层边界与600秒 package 上限。必须记录实际13节点结果，不能用旧失败或定向结果替代完整验证。

原失败 attempt a1f426…仍完整保留：10/13通过、MCP计数实际失败、repository实际signal7、verification明确os error28。Bus error因果仍未知，若重现必须诊断，不能跳过或归因磁盘后忽略。

## 验收与交接

当前 native actionExplanation 报8项 issue；11场景仍未验证、intent alignment未解决、用户收益尚未声明、成功canonical verification缺失。原生 Outcome 原文见 outcome.zh-CN.txt。本轮只落实授权的宿主缓存清理及 native amendment，未进行 PR、Ready、merge、finish、archive或close。

下一步：取得上述精确绑定的真实 confirm_review 并通过原生记录；在当前准入允许后应用已批准两行修改并提交，刷新最终材料绑定交Raydot真实审阅，再执行正式验证。聊天中将单独显式交付此黄色 Outcome；宿主是否已显示完整证据文件的状态未知。
