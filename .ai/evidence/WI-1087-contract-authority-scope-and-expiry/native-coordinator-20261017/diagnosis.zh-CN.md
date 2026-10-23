# 原生完整编排：真实失败记录

本轮源码 HEAD 为 87643464e733ca9f65657659779e8f76c10bf3f3，5 个产品文件 hash 与启动时一致。

用户明确 900 秒上限适用于验证执行节点；Raydot 技术决定采用原生完整串行编排。原生计划有 13 节点、每节点 600 秒、workers=1、nodesReused=0。CLI 无已核实的 resume/nodes 入口；现有 profile 对这些子命令返回 command_not_profile_verified，不能复用旧 11 条进度日志。保持 Contract、断言、验证范围及节点预算不变，移除了聚合器外层 timeout890；这不是提高节点期限。旧两次 124 及旧解释原样保留，此处更正其预算语义与停止建议。

工具持久会话中的本轮完整结束，外层耗时 921.069 秒，原生 route elapsedMs=919673，exit1。13 节点全部执行、0 复用、最大并发1，12通过、repository失败。全部 timedOut=false。CLI 节点 308.337 秒、repository 498.329 秒（exit101）、verification 82.747 秒（通过），其余10节点合计28.189秒。

原生自动生成 execution_failed attempt 72fb8a020a1f127da1126a5320ae0dd1c65129c3447c8e712a45c75420980713。它是失败证据，不是成功验证收据。repository stdout 被生产者截断为前65536字节，该标志保留；stderr 7326字节完整。失败用例为 linked_worktree_cargo_test_binary_embeds_its_own_manifest_directory，verification_service.rs:415 的内层 run.receipt.passed 断言为假。该断言不输出内层 Cargo stderr，当前不认定根因。

夹具会移除 CARGO_TARGET_DIR；源码默认回落 HOME/.cache/ai-cockpit-verify-target/<checkout hash>。当前 HOME=/home/agent，.cache存在，目标叶目录不存在。这是待验证的缓存权限线索，不是已证明的原因。下一步经用户批准以隔离无依赖Cargo夹具采集原始stderr，再精确复测失败用例。不能改断言、改HOME、宽泛chmod或扩大父目录授权。

CI 39006792511 对应该源码HEAD：quality已失败，route与v1-behavioral-oracle通过，Windows仍在运行。gh log-failed因整轮尚未结束而不返回日志；已完成job的REST日志重定向后403，未绕过。check annotation仅为Process completed with exit code 1，CI具体原因尚未知，不能假定与本地相同。一次性签名日志URL只留本机scratch原始观察，不提交Git；此目录只记录不含签名的403摘要。

原生Outcome已返回黄色，正式成功验证仍未就绪；admission allowed、blocking=false、humanDecisionRequired=false。没有伪造成功、没有合并/关闭WI或进入P2-P4。用户随后明确批准先正常commit/push保存失败证据；未额外调用CI取消操作。此文件是agent:Codex的事实说明，不是Runtime receipt或真人决定。
