# WI-1086 首轮完整正式验证（失败保留）

正常已安装 Runtime 5307ba53 在源提交 8ea965d8 上执行 Contract 声明的 workspace 验证，13/13 节点执行，10 通过、3 失败；外层耗时 610.901 秒、900 秒上限、未超时。原始 stdout 中保留完整结构化输出，独立 verification-attempt 为真实原生失败回执。没有成功 verification.json，不能称全量通过。

失败：CLI scope_amendment 1 条及 repository lifecycle_entry 2 条测试在 amend 后未先获取当前 preflight 就写新批准，触发已批准的 not_run 拒绝规则；修复这些旧测试的两个路径尚未纳入 Contract，最小提案在相邻 fixture-scope-proposal-20261016，未应用。composition interrupted-owner 原轮出现 procfs.cwd errno13；同源码单条复跑通过，根因未证实，原轮保留。

格式检查通过；此前 focused CLI/MCP 18、B 投影 2、不连续 Contract 链 1 的通过记录在 implementation-fix-20261016。focused 与当前失败全量严格分开。当前状态 checkpointed、准入仍允许 run_verification、humanDecisionRequired=false；intent_alignment_missing 和 user_visible_benefit_not_declared 未补造。原生 Outcome 仍报告 verification_evidence_missing，这不能覆盖失败 attempt 的实际事实。

工具链 rustc/cargo 1.98.1，构建线程与测试线程为 1，隔离 target；候选二进制未安装为共享 Runtime。元数据中的二进制 SHA 是实际现有候选字节，fmt 和 cargo --lib 等动作不声明生成了新 CLI。原始日志可能含工具固定尾空格，保留原始字节。

Owned subreaper 在本轮结束回收 849 个后代；结束后 PPID1 僵尸仍为继承基线 21,316，未批量 kill。末次工作盘约 688 MB，未清理缓存或旧资源。后续完整重跑前需重查空间。
