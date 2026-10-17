# WI-1085 第二轮完整原生验证：真实失败

Outcome: 🟡 已真实执行全部13节点，12通过、1 repository失败；不是正式验收。

## 决定与执行身份

Raydot实际新材料决定accept_permitted_unknowns已原生记录，request661c2ccf…、receipt9dc8bd00…，assurance=self_declared，仅原两项syntax Unknown。执行前确认humanDecisionRequired=false、blockers=[]、run_verification获准；Raw Unknowns与其余验收未知仍保留。

源码HEAD ef0a835486f2b1cdfd03a0f466e990ee1addd6ed；Contract7fea3f7d…、repositorySnapshot8a858ff1…；Runtime为保留的release二进制，actual digest0ffdcd20…，来源仍是已记录的固定源码a12ee2ff…，不是旧安装字节连续性声明。

Native attempt sha256:88ebd79aefea2bd1d09a69ba0a8d081e492ddc0315091ec9752f3df05dfb39fa 于2026-10-16T13:21:40Z（JST22:21:40）生成execution_failed。实际13节点执行、12通过、1失败、0复用、0超时，maxConcurrentProcesses=1，elapsedMs=872836，executionElapsedMs=870725，外部工具与Tini返回1。自然完成于890秒边界内；本次未测试全局强制超时回收路径。

## 验证发现

MCP节点通过，捕获输出逐项确认 material_review_plan_tool_uses_shared_read_only_repository_service 与 initialize_and_tool_list_expose_native_usage_record_without_hidden_helper 均实际通过，两处31→33回归被真实执行。verification节点亦通过。

唯一失败：resource_lifecycle_module_uses_explicit_imports，在crates/cockpit-repository/tests/resource_finalization.rs:36断言 !source.contains("use super::*")。新增legacy_source_lock_tests模块在resource_lifecycle.rs:11180使用通配导入，加入该模块的历史提交为54e04e46c27aeb922412b859a111b47041b6c3ef。这是断言失败，非链接器signal7或磁盘os error28；本轮未重现前两种错误，仍不证明先前Bus error的因果。

失败后Cargo停止执行repository的后续测试目标；不得宣称整个repository所有断言已通过。成功canonical verification仍缺失。断言原文、native stderr与完整受上限约束的captured streams均保留。该源码文件已在当前scope及implementationPaths内，最小后续修复计划为只展开上述cfg(test)模块的六个实际名称，保持守卫与所有业务断言。此失败证据提交不应用修复；后续实施另有提交、材料绑定和审阅。

## 资源与回收

启动可用11,880,263,680字节，高于11,211,053,176字节规划阈值（含2GiB预留）；结束可用6,474,584,064字节（约6.03GiB），全过程观测高于2GiB预留线。未再清理缓存、改target、profile、worker或断言。下一次验证必须重新估算增量；当前余额低于原“全部51变体再建”保守阈值，不能直接沿用旧thresholdMet=true。本轮已经建成全部本次native profile编译产物；后续若改源码，应考虑按相同package参数的--no-run预热，并单独记录其不是测试通过，以给900秒边界留出余量。不得增加900秒上限。

即时基线与结束均1425僵尸，按PID+startTimeTicks无新增/丢失；起始live转Z新增也为0。Tini根927618实际退出，same-session与已采样owned live/zombie均无残留，未见活跃cargo/rustc。Tini日志记录1233次收养僵尸回收、main child正常返回1与No child to wait。之前1419→1425的6个变化发生在观测窗口之间、本轮启动前，来源未归因，单独保留。

17份过程快照与前后快照均无未解决读取错误；瞬态ENOENT均经确认PID消失才标记resolved。采样不能枚举每一个历史后代，限制与原始Tini日志一并保留。

## 字节与限制

26份stream从原生attempt hex字段逐字节解码至短路径streams/cockpit-*.out/err.log；长度、SHA256、原生字段历史引用和truncated标记见streams.json。repository stdout为native 64KiB上限截断，其他stdout及所有stderr未截断。没有补造缺失尾部。原生attempt、第一次失败a1f426…与更早73a36…拒绝证据不覆盖。

## 下一步及验收边界

先提交推送真实结果，再仅修复现有scope中的测试模块导入，刷新实际材料请求交Raydot新审阅。不得放宽resource_finalization守卫、修改其他业务断言、复用旧材料批准或把12/13解释为通过。11场景仍未验证、意图对齐未解决、用户收益未声明，仍有5项治理issue。原生Outcome与validate输出保留；本轮未进行PR、Ready、merge、finish、archive或close。WI1086唯一清理收据仍仅在保留主检出原字节保存。
