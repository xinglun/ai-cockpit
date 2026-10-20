# 执行记录：WI-1090 根因修复

Plan: ../root-cause-proposal-20261020/CONSOLIDATED-ROOT-CAUSE-PLAN.proposal-only.zh-CN.md
Base: 0536a3541a00f4eae62b487ccb3382a4d0362174

- Ray 精确绑定真人确认已原生 controls 记录；无材料风险接受。随后 admission 已直接允许 run_verification，多余 preflight 被拒绝，保留真实输出，不重复重试。
- 接口检查：CLI/MCP 需在只读捕获后规划、plan-only 返回后共享 prepare，完成调用共享绑定 writer。核心 writer 原先无锁；新增临界区不递归获取锁。
- Ruling: 使用现有 WI Agent evidence 目录作 ledger，不创建额外管理目录；保留所有执行历史不删除。已批准 Contract 明确只跑六项检查，不扩大到完整 workspace suite。
- Ruling: 用户选择当前 agent 内连续执行；最终做独立时段自审，不另派子代理。
- RED CLI 初次因参数 -c 未用 --args=-c 被解析器拒绝，为测试输入错误，修正后重新执行，不能作为功能 RED。

- Task1: 真人确认原生记录完成；新绑定70ce/3798/7b1e不变。
- Task2: 共享捕获/短锁 prepare/完成复核、真实原生 S 归属、并发 CAS、legacy 覆盖拒绝已实现。CLI 首轮缺失事实 RED→GREEN；补充 rejected plan-only 写 attempt 缺陷 RED→GREEN。MCP 1/1、核心7/7已执行通过，包括真实原生前项归档→后继恢复、失败→成功→证据提交→自身 finish/archive。合成材料原先不是合法 Rust 触发真实 Unknown，改为合法 Rust 夹具后正向流程通过；没有绕过材料门槛。
- 三语6条流程体验标准按已批准补丁应用；原三份自身页面同步描述37范围和六项检查。当前真实工作项正式验证与自身 finish/archive仍待候选 Runtime。
- Ruling: gate 回归直接调用现有未改 gate 并读取完整 native fixture 的 Contract/Summary，不构造额外 ownership 投影。

- 小阶段验证：core-complete-regressions7/7；CLI（包含 rejected plan-only）1/1；MCP（包含 rejected plan-only）1/1；实际 gate 直接读取 native fixture 0/2/3 路径变化检查1/1；fmt --check通过。exact8-and-root-scope审计通过（8/2/2/27/2/19，额外 Rust 仅已批准7）。
- 实现仍在已批准范围；没有新命令、controls 类型、协议 schema、CI/gate 政策或全局配置修改。完成事务失败后已证明回滚保留启动事实，首轮失败原字节未改。
