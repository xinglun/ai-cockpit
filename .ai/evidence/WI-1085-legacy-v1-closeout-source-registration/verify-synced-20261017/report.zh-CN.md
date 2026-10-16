# WI-1085 同步后正式验证、CI 失败及最小修复交接

Outcome: 🟡 当前修复已完成 lint/格式检查；新材料待 Raydot 决定，修复后正式验证和 Windows 四例仍待实际执行。

## 本轮真实完成

Raydot 对 request 88caadcf… 的真实 `accept_permitted_unknowns` 已通过 Runtime 记录为 `sha256:80487dba72b48a455d91e72d876137be6ebbb9b7eced3d9e641cc96306b85472`，reviewerActor=agent:Raydot、assurance=self_declared。`decision-relay.json` 明确由 Codex 记录父线程转达的实际审阅。第一次因 authoritySource 未逐字匹配 Contract profile 被原生拒绝，失败 input/output/error 保留；随后读取既有获批 profile 的精确文本，未改 Contract/门禁，原生记录成功。该收据不自动接受之后的源码变更。

在 source HEAD `9e1e478682089ab96e8df5c2a8779fdfb24782ea`（实现 2dc2826f…）上完成一次全新原生正式验证：13 planned/executed/spawned/passed，0 failed/reused/timedOut，maxConcurrentProcesses=1，869792 ms。外围 Tini/timeout/tool 实际 exit0，未触发 890+10 秒边界。

- attempt：`sha256:2199e77683da7edf443834f0ce239e3144908669102e9f5c96f017dde1d510ce`
- canonical receiptDigest：`sha256:fb9636cf10149ecae2980e90588282b607e64ceb7b7506aec64f4bf06c478af0`
- Contract：`sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1`
- repositorySnapshot：`sha256:373a76c43e857ba33189221b89e3d875cd2c5dad534bd01fc619599a45810d4b`
- Runtime：`sha256:0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4`

canonical 与 attempt 的 receipt 完全一致。正式结束 Native status 是 verified/fresh、blockers=[]、humanDecisionRequired=false；源码后续已变化，现时结果不再标为新源码 fresh。`canonical-before-lint-fix.json` 保留原 canonical 字节，旧失败 attempt 全部 hash 未变，26 个 stream 的 UTF-8 捕获字节、hash、truncated 和真实原生字段引用在 `streams.json`，不补造截断尾部。

准备阶段 13 个 --no-run 包命令实际 exit0、执行测试断言为0，与正式 13/13 分开。prepare 的外层工具 poll 曾报告1、shell/Tini/13包记录0，差异原样保留；正式运行改为直接 exec Tini，实际工具 exit0。未为此另建 runner 或重跑旧 head。

## CI38941726963 的实际失败

CI head=`9e1e4786…`，实际 checkout PR merge=`9b27b0ae14a77f8573db507ee3a85776289dc807`，base=`9e370e1c…`。Windows artifact12064756118 的原字节 binding 保存在 `ci-windows-revision-binding.json`，下载 ZIP SHA256=`16eed85ad0b3ac4e8282bb4de70fd8c17cd54acedac3b61bed51ee6ab46d763c`。

quality job117112862119：parity 和 format 已过，workspace_clippy 4处失败（lifecycle type_complexity、两处 legacy unnecessary_unwrap、resource lock bool_assert_comparison），package_tests 被阻断。官方 diagnostics artifact12063628421 ZIP SHA256=`7e6f83709a6958376bcb3ee7fb810d630e41bd4f7ebea4285f6d9dceb9a9f000`，原始日志保留。附带的 coordination 日志真实 passed，不把它冒充该失败或本 WI Windows 覆盖。

Windows job117112862223：133 passed / 3 failed / 0 ignored，实际单测执行825.73秒，非超时。新四例中仅 `pinned_repository_root_rejects_windows_reparse_alias` 真实成功；其余 history_reads、obligation_enumeration、publication 在 helper2807创建junction失败，stderr=`Parameter format not correct - "decisions"`，尚未进入安全语义断言。四行原始输出及精确 source 绑定见 `windows-case-result.json` / `ci-windows.log`。运行中读取日志得到BlobNotFound也单独保留，未将观测失败当作空结果。

## 统一最小修复及实际 GREEN

等本地正式验证结束并核对回收后才编辑源码。改动提交51d1fede…与007fdcba…，此前未逐条推送或触发全量 CI。

1. lifecycle 增加私有 tuple 类型别名，函数返回的实际类型和业务逻辑相同。
2. legacy 两处改用借用的 if-let Err，保留分支、错误诊断、安全断言及返回行为。
3. lock 测试 `assert_eq!(result.unwrap(), true)` 改为等价 `assert!(result.unwrap())`，Raydot 已明确批准，不降低断言。
4. Windows helper 仅把 link/target 的 components 收集成原生 PathBuf 后传给 cmd.exe；保留 OsString，不 canonicalize 被测 alias，不跳过 junction 创建失败，所有 reparse 属性、读写拒绝、absent canonical 与 victim 断言不变。
5. 首次完整 workspace Clippy 清除四处 repository lint 后新发现 MCP1137 `collapsible_if`。该路径已在当前 scope 内；折叠等价 Some(sourceRepo)+非法路径条件，保持原错误字符串和 None 行为。actual exit101 原始记录保留，没有加 allow。

source HEAD=`007fdcba2e81d01285a694dce57cd78e0f74fcf4` 的第二次完整 `cargo clippy --locked --workspace --all-targets -- -D warnings` 实际 exit0，42.69s；`cargo fmt --all -- --check` exit0。Clippy/格式检查不执行 Contract 声明的完整测试，不能当作新正式或 Windows 收据。

## 新材料精确绑定（只读生成，未自行接受）

- Work Item：WI-1085-legacy-v1-closeout-source-registration
- repositoryId：`sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`
- reviewedSourceHead：`007fdcba2e81d01285a694dce57cd78e0f74fcf4`
- requestDigest：`sha256:10f95e4ec0a84e26ab0ccd9f240c515217340b0bef949a92f144d3005a6bc663`
- Contract：`sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1`
- sourceSnapshotDigest：`sha256:d769eda16f0a78a672715212f5bb4820b4a443880eb197f5eabcc465e7852871`
- materialManifestDigest：`sha256:b17c12d60094a36f04cfd9ba4b5895dd7e92c38c5dcb361aa87eef771afc82cf`
- repositorySnapshotDigest：`sha256:9a97106cf5ce0a70f760d4837d554570e1fd7792a3cce29280c32f054a77e21b`

36条、0Finding，32条与已接受88请求完全一致，仅 MCP/lib、legacy/src、lifecycle、resource_lifecycle 四个现有scope文件改变；仍仅原legacy源码/对应集成测试两路径 readable_committed_rust_syntax_unknown 可审阅，集成测试未变。完整 request与差异在 `final-material-request.json` / `final-binding-for-raydot.json`。中间64f请求因随后MCP修复已过期，不作为当前审阅目标；旧88决定不复用为新请求决定。

## 状态、边界与下一步

当前 Native verification=not_ready、evidenceFreshness=stale_or_invalid、blockers=[]、humanDecisionRequired=false，safeActions允许原生材料决定和验证。11场景按新源码仍unverified，intent unresolved，用户收益未知；降低错误释放scope风险仅推断，未做真实WI-1004登记。原生Outcome完整返回在 `final-outcome.zh-CN.txt`；没有主张其已经获host显示确认。

A6仍unverified：必须等修复版本四个本模块Windows实际成功行。四例限定directory junction/reparse路径，不是所有reparse类型或末端文件symlink的全覆盖；其他模块通过不能替代它们。新源码完整验证须在新材料真实决定后按Native准入执行，不复用旧13/13。

正式进程窗口前后僵尸1439→1439，无新增/丢失身份，Tini根消失、同session无live、cargo/rustc无残留，Tini记录1582次回收。后续clippy1窗口1440→1440、clippy2窗口1441→1441；窗口之间的全局增加未归因于这些命令，观测限制保留。可用空间始终高于2GiB，最近约6.3GB；没有扩大删除或清理。原Runtime、旧失败证据、主checkout独有WI1086cleanup收据继续保留。

下一步：将这些修复和真实证据一次推送，保持PR1044 Draft并启动当前源码CI；把本请求交Raydot实际审阅。当前生产head的CI已completed/failure，记录推送不会中断它。新CI运行号由推送后的GitHub事实返回，不预先编造。普通技术审阅交Raydot，无须重复找Ray；明确真人门禁仍保留，没有Ready/直接合并/archive/close/额外资源清理授权。
