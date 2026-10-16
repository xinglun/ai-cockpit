# WI-1086 两个旧测试文件的最小扩围提案（未应用）

## 真实 RED 与原因

源码 `8ea965d8b21cc908f3cca3941af411807105808c` 的正常 Runtime workspace 验证执行全部 13 节点，10 通过、3 失败。CLI scope_amendment 的 1 条测试和 repository lifecycle_entry 的 2 条测试使用旧顺序：typed amend 将 Summary.preflightState 置为 not_run，测试随后直接把当前 Contract 与保留的旧 preflight 字段组成新的 confirm_review。当前校验器按已审阅要求拒绝此写入，错误为 `preflight decision evidence requires a recorded green, yellow, or red preflight result`。

这三条均为真实回归失败，不能用 focused GREEN 或旧 CI 代替。lifecycle_entry 的 first_typed... 测试第一次 preflight/确认可行，后续 sourcesAppend 之后的确认才遇到同样旧顺序。

## 范围

只提议将 `crates/cockpit-cli/tests/scope_amendment.rs` 和 `crates/cockpit-repository/tests/lifecycle_entry.rs` 加入本 WI scope 和 implementationPaths。附在内存中生成、尚未应用的最小 diff：在三个 amendment 后确认点，先调用现有真实 preflight，再读取其 Summary/构造测试批准；原来的确认后 preflight 与所有断言保留。产品文件不再扩大，尤其不放宽 governance_controls 校验器，不写人工 Summary，不重绑定历史批准。

## 兼容性与验收

生产协议、receipt schema、同 Contract 有效批准复用均保持。原 3 条测试必须通过；现有 CLI/MCP 18 条、阻断事实投影 2 条、重哈希不连续 Contract 链 1 条均不得退化；正常完整 workspace 验证须重新产生真实全通过回执。missing/unknown/not_run、旧 Contract/mixed binding 必须继续拒绝且拒绝时 Summary/receipt 字节不变。先测试 fixture 最小面，再完整验证，单任务、隔离 target、命令上限 900 秒。

## 其他失败

composition 的 interrupted-owner 测试原始全量轮失败：fresh pid 的 procfs.cwd 为 PermissionDenied/errno13，保留 cleanup Retained。相同源码单条复跑 1/1 通过，尚未证明根因，不更改其源码或断言，不将复跑替代全量失败。

## 准入与停止边界

此目录仅供 Raydot 技术审阅；request 为待校验的 proposed typed-amend 文档，未调用 amend、未修改这两个测试文件。批准后须先读取当前 Contract digest、使用现有 typed amend、再查询实际 Runtime 准入；如 Runtime 需要新的真实绑定决定，报告精确请求，不能借旧字段或漏洞解锁。当前已记录的同 Contract 真人批准保留，不伪造或重复记录。旧 WI-1085、Sentinel、共享 Runtime、Ready/merge/close/清理不在范围。
