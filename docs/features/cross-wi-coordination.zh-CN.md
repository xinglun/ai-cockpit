# 跨 WI 协调能力

此能力把依赖声明、影响报告、安全边界协调、精确组合验证和人类 Outcome 投影
连接成闭环。它按 WI opt-in，不会把“请求确认开始”变成额外治理门禁。

repository service 是领域边界，CLI 和 MCP 只是适配器，Outcome 是投影。查询保持
只读；登记、影响、协调状态迁移、恢复消费和组合验证都是显式写入。

支持范围是同一 Git common directory 下的 linked worktree。固定 Runtime 负责生命
周期兼容性；候选 Runtime 必须声明 collaboration capability 后，才可以写入或消费
这些记录。

登记会绑定并重新核验 Git 身份、active Contract、head、branch 和 regular evidence；
组合记录写入 Git common directory，由有界验证执行器运行，并通过 Outcome 暴露真实的
逐节点复用与清理事实。
