# 实际四文件有限材料审阅

Reviewer: agent:Codex；assurance: self_declared；完成记录时间 2026-10-20T03:38:36.417132+00:00。

本报告是实际Agent静态审阅证据，不是扫描器完整性证明、真人批准、正式测试收据或托管CI。真实风险批准来自Ray2026-10-20T03:28:22Z Sentinel_4894c6afcf14819186152a312ace0394；真实Contract确认来自Ray2026-10-20T03:34:25Z Sentinel_f5269f1600248191b01a5d3d8fb3cd8a，均有原始relay和native记录。

实际请求 `sha256:f98cf3c3e5146c83cf10afd3c1046c9753fa240dd094d592235059fb5e8f4b6a`，Contract `sha256:be0f85d58e29bcc709fde3a3a81d1e221afff96bb8989783d21acd649e42935d`；manifest `sha256:76009df3df2d11067ad3e78261ba55899349dce0046866e0723da893f724525b`；material sourceSnapshot `sha256:25b60b47c1e677df3b9174dbb61689be6f37841c7febb1828cb8839e69433e09`；reviewedSourceHead `6048fdcc2a8acb3da373ab496cc56efb5f3732ef`；analysisImplementation `sha256:ccf952067cbe6eba3dfa116f7a3d87293d69f6c7ddb5747953157ff63493cc83`；profile `sha256:711819ed3ca489e577a165080e3c305ee366cef0f87eed8580bed46ba4e18cf8`；effectivePolicy `sha256:4022dbdd989b6b05ee6116c7fc73b9465a373c1ed42b01999f77ea65d8d8921a`。固定scanner rust-material-v1 / cross-platform-rust-source-v1。四个Unknown成员逐一与真人批准的blob/hunk及当前磁盘原字节核对一致，无新成员或Finding，均readable_committed_rust_syntax_unknown且reviewable=true。

| 精确成员 | after blob | changed hunk |
| --- | --- | --- |
| `crates/cockpit-cli/tests/verify.rs` | `sha256:2bc8aeed5b4f5ede41bb2eb7ed0167261368bab9afd3c9ee93cde0da730bbb84` | `sha256:27441ef578adf4d56db736e2b96f15d2ca5e52376dc9e963e668309855e7a249` |
| `crates/cockpit-mcp/tests/rpc.rs` | `sha256:1b4ddbe9d24b98e24920808f913674ee01ba722f3ee429221d6da4847700738b` | `sha256:8c3a9e5bd225388cf57c28f78d49ac31fd7c5f4afddaba078cfeb7a821921ea7` |
| `crates/cockpit-repository/src/material_review.rs` | `sha256:0ebba06d433ff1aa1a25fbc70d59d8735575958142438c89a813c6381d571a48` | `sha256:09435234cf7961bdc1248e876d9215cc57a5c61601c889f6946bcd43076c2e60` |
| `crates/cockpit-repository/tests/lifecycle_order.rs` | `sha256:f971e470e9a4092f238dfa8fc8792ebf475c0fc9c707d27ac019f9ccf6a76d2d` | `sha256:9b6e660e749b97893fedf74ecfbf8387afd7b7e1766e67c9cf07b59e590c7f96` |

实际检查当前变更与相关上下文，包括：

1. material_review.rs 的请求/决定验证、严格当前receipt读取、有限authoritySource历史证明与H/P/C0链、no-follow有界读取、源HEAD/成员身份/目录身份绑定、sealed prepare及Summary字节CAS、提交前重验、错误后实际状态读回和对应隔离测试。旧receipt不是新authoritySource的当前证明；只有新真实typed决定产生当前证据。未发现本次有限审阅范围内明确的注入指令、跳过Findings的通路或实际live授权调用。继承源码原字节未改。
2. lifecycle_order.rs 的原生前项→后继责任来源、真实门禁调用、等待锁时漂移拒绝、准备/完成CAS、失败保留与正常证据提交后收尾、canonical材料Unknown及同秒新失败负例。人工样本明确标记synthetic-fixture；手工篡改/删除仅为TempDir中的负例，不是当前仓库治理记录。新增可读字符串构造用于生成合成语法Unknown材料，所审源码和测试意图一致。
3. cockpit-cli/tests/verify.rs 的新fixture在tempfile目录运行固定参数Git/CLI/Python，实际先检查plan-only只读、失败前路径可见，再观察材料阻止finish及Red Outcome。脚本是测试断言，未把样本收据当成本工作项证据；没有新增网络、真实grant或共享配置写入。
4. cockpit-mcp/tests/rpc.rs 的新fixture使用TestTempDir、支持的MCP verify/status/Outcome表面和原生库finish guard；未虚构MCP finish工具。失败attempt、canonical缺失/有效、原生材料边界和host文本断言保留；其合成Rust Unknown字符串只属于fixture材料。

有限结论：当前精确四成员可在已授权范围内记录accept_permitted_unknowns；这是对可读语法分析Unknown的有限Agent判断，不是通用安全结论。本报告未运行额外测试，后续只执行当前Contract声明的六项正式检查。独立技术审阅由Raydot父线程报告通过，另有relay证据；它不被冒称为本人的外部审阅或当前正式测试。

残余风险：扫描器对这些可读Rust语法仍无法完整判定；静态人工/Agent审阅不能排除遗漏、全部竞争窗口或平台实现差异。Windows托管运行、实际消费者和Sentinel均未验证。本次仅接受上述固定blob/hunk的扫描盲区，不扩大到新成员、Finding或变化身份；发生其一立即停止并按新实际绑定处理。self_declared不升级为human/provider/release或scanner-complete保证。原始scanner Unknown和历史失败继续保留。
