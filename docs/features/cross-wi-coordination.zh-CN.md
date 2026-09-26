# 跨 WI 协调能力

此能力把依赖声明、影响报告、安全边界协调、精确组合验证和人类 Outcome 投影
连接成闭环。它按 WI opt-in，不会把“请求确认开始”变成额外治理门禁。

repository service 是领域边界，CLI 和 MCP 只是适配器，Outcome 是投影。查询保持
只读；登记、影响、成果发布、协调状态迁移、恢复消费和组合验证都是显式写入。成果
发布绑定当前登记代次和精确证据字节。

支持范围是同一 Git common directory 下的 linked worktree。已安装的 Runtime `0.2.113`
负责生命周期兼容性且不会读取协作存储；候选 Runtime 必须声明 collaboration capability
后才可写入或消费这些记录。不能假设旧二进制会读取或执行候选版本新增的 Contract 检查覆盖字段。

登记会绑定并重新核验 Git 身份、active Contract、head、branch 和 regular evidence；
组合记录写入 Git common directory，由有界验证执行器运行。只有实际观察到的
executable、命令、有效环境、声明输入字节和依赖 receipt 一致时才复用；节点输入未知时
禁用复用。Outcome 分开展示适用性、真实合并、清理和复用事实。

组合验证中断时会持久记录活动 verifier 进程组。只要该进程组仍存活或状态未知，重试就
会保留临时 worktree。Unix 还会检查同一用户的进程 cwd 和 worktree 下的打开文件句柄；脱离
session、仍引用该树的后代同样会阻止清理，检查不完整时 fail closed。Windows 则由 bounded
executor 的 kill-on-close process job 管理后代。Outcome 只有在终态记录内部一致时才展示
通过或可复用检查。

组合覆盖来自 digest 绑定的必需 `verification` 检查，可用 `coversScenarios` 或
`coversConstraints` 声明语义；调用方标签不能授予覆盖。执行仓库必须与协调存储属于同一
Git common directory，因此复制 repository id 的独立 clone 会被拒绝。
