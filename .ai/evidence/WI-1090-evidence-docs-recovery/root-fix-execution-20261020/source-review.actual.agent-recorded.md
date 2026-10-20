# 根因修复独立时段自审（agent:Codex）

审阅范围：0536a3541a00f4eae62b487ccb3382a4d0362174..4237f5ec394c40a62b39b9b4ed8d2ee0fd9140fe。依用户选择在当前 agent 内连续实施，自审不冒充独立真人或外部审阅。

- 4实现/3回归/3流程文件及原自身三语投影符合已批准范围；未改 gate、协议 schema、全局配置或继承 Rust2。
- 起点复用原生 prepared/committed/journal 验证，检查 repository/owner/祖先；S 与仓库 B 分离，scope 不作路径事实来源。实际正式 Summary 中原生 S=cba3，三 parity 在检查前可见，继承 Rust2 未归本项。
- 只读捕获先于路由；plan-only 成功与拒绝均不登记观察或 attempt。prepare 和 completion 各持短锁、require_subject、复核源/Contract/配置/策略/HEAD/journal。checks 不在锁内。并发旧计划通过 generation 比较拒绝，旧 raw writer 不覆盖新观察。
- 失败保留事实；完成事务回滚基线含检查前事实。成功使用绑定路径，正常 `.ai` commit 不把历史 HEAD 当永久条件。合成原生失败→成功→evidence commit→finish/archive已通过。
- 正式6/6，复用0、最大并发1、9个匹配用例。当前实际 finish/archive、材料 Unknown 及托管CI仍独立评估，不将 synthetic 或 source-verified 代替整个交付终态。

本次自审未提出新的源码修改。验证覆盖为 Contract 声明六项，不宣称跑过完整 workspace 或 hosted CI。
