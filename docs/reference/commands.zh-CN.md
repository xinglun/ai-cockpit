---
author: AI Cockpit maintainers
title: "命令参考"
description: "当前 CLI 命令面及其修改或 evidence 边界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_commands
---

# 命令参考

现在 `close` 要求当前 finalization head 的 disposition 必须是 `deleted`；
`retained`、`blocked` 或 `unknown` 的 head 会在写入 close decision 前停止。对于旧
Runtime 已经产生的不可变记录，`work-item finalize` 允许在 close 后追加一条严格绑定的
deleted transition，作为有限的历史 reconciliation。该 transition 必须绑定已关闭 root
的路径和 digest，并作为 append-only cleanup observation 验证；它不会重写 close receipt，
也不会让新的 Work Item 采用 retained close。

`work-item finalize` 将首个 receipt 写入 `.ai/decisions/<id>.finalize.json`。其中 PR base 必须等于归档 Contract 不可变的 `baseRevision`；记录与 `finalize-verify` 都会拒绝不一致，包括 sequence 0，绝不会把该链报告为 verified。归档前 rebase 要刷新 active Contract 绑定；归档后必须走 recovery，不能改写 receipt 或 archive。若该不可变链根已存在，typed transition envelope 必须绑定唯一 head 的 predecessor digest 与下一 sequence；Runtime 追加 `.finalize.<digest>.json`。`finalize-verify` 返回 `headPath`、`headDigest` 和 `sequence`，`close` 会绑定这些值。当 receipt commit 推进了全部对齐的 head 时，sequence-1 merge observation 还可以绑定 `governanceAppendRevision`。Runtime 要求祖先区间只有新增；除同一 Work Item 的普通 finalization receipt 外，唯一允许的 evidence 新增是完整的固定 schema 文件对 `.ai/evidence/<id>/quality-route-post-finalize.json` 与 `.ai/evidence/<id>/repository-gates-post-finalize.json`。每个路径必须是 `A`-only、`100644` regular blob，且其归档 Contract、PR revision、route digest、manifest、profile 与 passing gate 绑定必须一致。这对文件是 evidence 而非 authority，不能替代仍然必需的 finalization receipt 新增；也不会授权任意 evidence 路径或归档修改。

所有 repository 命令都接受显式 `--repo <path>`。产生记录或 decision 的命令在 stdout
保持 JSON。`finish`、`archive`、`close` 默认还会在 stderr 输出本地化的面向人交接；
其 `--json` 只抑制该 handoff。`work-item outcome` 默认在 stdout 输出本地化的面向人
交接结果，需要稳定机器接口时使用 `--json`。失败或 unknown 不能算 pass。

| 分组 | 命令 | 边界 |
| --- | --- | --- |
| 只读 | `inspect`、`observe`、`status`、`compatibility`、`migrate plan`、`capability show`、`diagnose`、`doctor` | 读取 repository 状态或 evidence，不静默修复。 |
| 派生投影 | `knowledge query` | 仅通过显式查询物化或复用仓库本地 `.ai/knowledge/` 索引；报告 `projection.writeBoundary=repository-local-derived`，不会改变治理权威记录。 |
| 准备 | `attach`、`profile confirm`、`profile propose` | 创建/更新协议状态、确认 profile，或输出只读候选。 |
| 迁移 | `migrate apply --approved` | 只应用经过审查的 repository schema migration，并写入绑定 Runtime 的 migration receipt。 |
| 治理 | `preflight` | 读取 Contract，返回 green/yellow/red decision 与 `reviewState`；不完整或不确定的 Contract 为需人工确认的 yellow，不能越过 checkpoint。 |
| Work Item | `work-item new`、`start`、`status`、`checkpoint`、`finish`、`archive`、`close`、`validate`、`controls`、`recover` | 读取请求级状态投影或写入显式生命周期记录；`close` 和 recovery 都要求显式 human decision。 |
| 并行 Work Item | `work-item boundary`、`work-item declare`、`work-item slot acquire|release|list` | 绑定 Contract 并行路径并管理 repository-local slot；unknown 时序列化。 |
| Verification | `verify` | 执行有界命令、记录 evidence，并可绑定 Work Item。 |
| 外部 evidence | `evidence import`、`evidence list`、`evidence policy`、`evidence purge-plan` | 将精确 provider bytes 绑定到 Work Item，声明有界持久化策略，或生成确定性的非破坏性处置计划。 |
| Audit | `audit export` | 生成绑定 repository 的稳定事件包交给外部保留方；不宣称本地 immutable。 |
| Adapter | `agent list/install/doctor/repair/detach`、`mcp` | 管理显式选择的 repository-local Agent adapter，或通过 stdio 提供 JSON-RPC；所有操作都绑定 `--repo`。 |

## 重要选项

- `verify --command <program> --args <comma-separated>` 执行显式命令且总是 fresh；`--work-item <id>`
  记录该 Work Item 的 receipt，但检测到的 Cargo/npm 命令使用动态的 profile-authorized 路径，显式自定义命令仍总是 fresh。
- 不提供 `--command` 的 `verify` 会检测 Cargo 或 npm，并可能使用已确认 profile 做跨进程 reuse。只有当前
  repository、snapshot、profile、Runtime、command、scope、stage、runner、base、toolchain、dependency 和 policy
  identity 全部精确匹配时才允许 reuse；否则执行声明的命令并报告拒绝/升级原因。耗时或缓存状态绝不会跳过 required/protected node。
- `verify --workers <n>` 要求正数并限制并发。
- `work-item boundary --repo <path> --id <id> --file <boundary.json>` 将可选的
  `concurrencyBoundary` 绑定到 Contract。四类路径和 `maxWorkers` 会被验证；后者是 slot 容量，
  不等于 `verify --workers`。
- `work-item slot acquire|release|list` 管理 `.ai/parallel/leases/` 下的独占 lease。lease 绑定
  repository 与 Work Item；缺失、格式错误、含糊或过期状态都会 fail closed，不存在全局 current Work Item。
- `start` 要求 `--id`、`--intent`、`--goal`；要得到 green governed flow 需要 `--authority authorized`。
- `start` 或 `work-item new` 之前，Runtime 会执行 repository-scoped 入口门禁。非 `.ai` 的工作区变更、detached HEAD、已发现的远端默认 ref 与当前 HEAD 不一致，或存在没有有效 close decision 的 archived Work Item，都会 fail closed；门禁不会改写 archived bytes。`work-item recover` 创建的 successor 是显式的同一修复链续接，不是独立的下一个 Work Item。
- 同一入口门禁还会拒绝普通 Work Item 使用 repository primary worktree 或已知 default branch。请使用 feature branch 上的专用 linked worktree。没有明确远端 default base 的 linked worktree 会被拒绝，不会被当作 ready；没有 linked worktree 的本地 calibration repository 仍保持 `status: unknown`，直到配置了可发现的 base。
- `work-item new --repo <path> --id <id> --mode <mode>` 创建 `not_ready` 骨架，只填充 snapshot-derived facts，
  人类字段保持空值或 `unknown`；过渡期 `start` 复用同一 writer。repository-local 独占 reservation
  会让重复竞争 fail closed：同一 ID 只有一个请求成功，另一个失败；不同 repository 仍然相互独立。
- `work-item outcome --repo <path> --id <id>` 按已完成内容、问题、停止、风险、未知、决定、验证、影响和下一步的顺序输出面向人的结果。
  自动化请使用 `--json`。状态标记和语言规则见[面向人的 Outcome](outcome-report.zh-CN.md)。Work Item 完成后还会绑定类型化的
  `*.task-report.json`、面向人的 `*.task-report.md` 和 append-only 的 `*.events.jsonl`；它们是绑定 evidence 的投影，
  不是额外的 authority，也不能替代 Contract 或 verification receipt。
- `finish`、`archive`、`close` 保持 stdout 生命周期 JSON 不变，并默认在 stderr
  渲染同一份已校验的人类 Outcome；机器专用输出使用 `--json`。`finish` 被阻止时，
  CLI 先输出已持久化的红色或黄色 Outcome，再返回原有 nonzero 错误，绝不会把失败门禁
  转成成功。CLI 无法强制宿主 Agent 或 UI 打开/展开对话面板；宿主必须展示 stderr
  handoff，或用 `work-item outcome` 确定性重放。
- `work-item status --repo <path> --id <id>` 是只读命令，输出生命周期、治理状态、活动健康、事实计数、阻塞项、未知项、evidence 和 source digest；不会调度任务，也不会臆造百分比。
- `work-item inspect --repo <path> --id <id>` 是兼容性、implementation approach 和并行 slot 的只读投影。
  它在内存中计算 approach，不会创建或刷新 `.ai/work-items/active/<id>.approach.json`。只有明确需要
  repository-local approach artifact 时，才使用作为写入边界的 `work-item approach`。
- archived 但没有有效 close decision 的 Work Item 是生命周期阻塞项，不是已完成项。其 `safeActions` 会明确剩余收尾：绑定资源的项需要 `finalize_resources` 或 `cleanup_resources`、`record_finalization`、`finalize_verify`，然后执行 `close_after_cleanup`（已验证为 Deleted 时可执行 `close`）；没有外部资源的项需要 `close_after_review`。Agent 必须按这些 action 执行，在 predecessor close 或显式 recovery 前不得开始下一个 Work Item。
- 顶层 `status` 还输出确定性的 `readiness` 对象。只有在命名分支干净、HEAD 与唯一发现的远端默认 revision 完全一致、没有 active Work Item 且没有等待 close 的 archived Work Item 时，`readyOnBase` 才能为 `true`。远端元数据缺失或含糊时为 `state: unknown`，绝不输出 green；`blocked` 会列出精确阻塞原因，`unclosedArchivedWorkItems` 会列出需要 close 或显式 recovery 的记录。
- `work-item status --repo <path> --all --json` 按稳定 ID 顺序聚合 active 与 archived Work Item，输出固定的
  green/yellow/red/unknown 计数、成员 diagnostics/digest、当前 repository snapshot digest 与确定性的 index digest。
  格式错误或 foreign 的成员会成为显式 unknown entry，其他成员仍然可见。这个动态 counterpart 不会写入
  `.ai/cockpit/work-items/index.json` 或逐项 status 文件；MCP 使用 `work_item_status` 和 `{"all": true}`。
- `capability show --repo <path>` 输出绑定 Runtime identity 与 repository 的 registry。观察到的技术能力、profile
  confirmation、repository binding、adopter acceptance 与 external ownership 是不同状态；仅有文件不能证明
  `adopter_accepted`，缺失、格式错误、过期或 foreign 输入保持 unknown。MCP 使用 `capability_show`。
- 重复执行 `observe`、`capability show`、顶层 `status` 和单项/全量 Work Item status，不会写入 tracked
  repository bytes 或 observer cache。
- `work-item validate --repo <path> --id <id> [--json]` 只读统一检查 Contract/Summary 的 scenario coverage、stable acceptance evidence、intent alignment 和可选最终维度 receipt。
  `work-item controls --repo <path> --id <id> --input <json>` 只记录显式提供的 projection 字段（包括绑定 identity 的 `decisionEvidence` review receipt），不能改变生命周期状态、Contract facts 或 verification receipt。
- `work-item recover --repo <path> --id <id> --input <receipt.json>` 记录绑定 identity 的 `retry`、`successor` 或 `supersede` decision。`supersede` 要求已经绑定的 successor Work Item，并把 predecessor 归档为明确的历史 `superseded` 状态；原始 bytes 不会改写。receipt 必须绑定 predecessor 的 Contract、Summary、Outcome 以及存在时的 event digest，并绑定当前 Runtime identity。既有 receipt 永不覆盖，后续 decision 使用 digest 后缀文件；recovery receipt 不会让 verification 自动变绿，也不会静默重写 predecessor。被替代的 predecessor 不是当前成功或失败，后续工作由 successor 负责。Outcome 与 archive consumer 会重验每个 current candidate 的 regular-file/文件名边界、repository 和当前 Runtime identity、predecessor digest、时间戳、decision shape 与 successor Contract 绑定。invalid 或 ambiguous candidate 以 `recovery_decision_invalid` 失败关闭；历史 archive bytes 与投影保持不可变。当 retry 的 predecessor digest 在新鲜归档的 Contract/Summary/Outcome/Events 中不再匹配时，它属于已消费历史，静态门禁会投影真正的 finalization 路径，不会虚构 recovered 终态；仍然匹配且 blocked 的 retry 继续 fail closed。
- `profile propose --repo <path>` 只读输出 `candidate`/`proposed` amendment，不会应用 profile baseline 修改。
- `agent list --repo <path>` 是只读操作；`agent install` 是唯一正常的 adapter 写入口，必须指定
  `--provider`（`auto` 只有在恰好一个无歧义安全 surface 时可用；`AGENTS.md` 默认选择 Codex）。`agent doctor --repo <path> --json`
  返回严格状态报告，并使用 0（verified）、1（degraded）、2（配置错误）、3（需要人工介入）退出码。
  如果 managed section 或 ownership record 被修改，`repair` 和 `detach` 会 fail closed；任何命令都不会写入全局 Agent/MCP 配置。
- `preflight --contract` 通常指向 `start` 生成的 `.ai/work-items/active/<id>.contract.json`。
- `work-item new` 生成的骨架状态是 `not_ready`。对它执行 `preflight` 会有意返回
  `yellow` 与 `reviewState: needs_human_confirmation`；补齐人工字段后必须重新 preflight 才能 checkpoint。
- `close --human-decision approved|confirmed|rejected` 是 human decision 记录，不是 verification evidence。
  `approved` 和显式的 `confirmed` 都是正向终态决定；`rejected` 不能把 Work Item 晋级为已实现。
- `evidence import --repo <path> --work-item <id> --metadata <metadata.json>
  --raw <provider-output>` 会用精确 raw bytes 的 digest 校验严格的
  `DelegatedEvidence` metadata，并在 `.ai/evidence/external/` 写入绑定
  repository/Work Item 的 receipt。`evidence list` 会重新验证这些 receipt；过期或撤销的
  provider claim 不会因此变成 authority。
- `evidence policy --repo <path> --work-item <id> --classification <value>
  --persistence <value> --retention-days <n>|--expires-at <timestamp>
  --disposal-action <action>` 写入严格 retention policy。`secret_prohibited`
  禁止 `full_capture` 和 `redacted_capture`；`digest_only` 不保存命令原始输出；
  `no_persistence` 在无法保存 completion evidence 时 fail closed。`evidence
  purge-plan --repo <path>` 只输出稳定的处置计划，不会自行删除 evidence。
- `audit export --repo <path> [--output <file>]` 输出稳定的 `AuditEvent`，包含 event ID、主题 digest、
  repository/Work Item identity 和 Runtime identity。manifest 会设置 `externalRetentionRequired: true`；
  输出文件幂等，只是交给 SIEM、WORM、S3 Object Lock 或其他外部保留方的 handoff。
- Task Outcome report 使用严格类型化 JSON。每条 claim 在可用时都绑定 evidence reference；明确标记的 inference 不能当作已验证事实。
  event stream 在 Work Item finish 时 append-only，并校验 repository/Work Item identity、顺序、安全 detail 内容和 evidence reference 边界。
  archive manifest 绑定 event stream 与 report JSON/Markdown digest；close receipt 会包含最终 report 及其 digest。
- 如需可审计决定，请增加 `--actor`、`--authority-source`、`--reason`、`--decided-at`，并可重复提供
  `--evidence-ref`、`--policy-ref`、`--resume-condition`。结果的 `structuredDecision` 写入
  `.ai/decisions/<id>.close.json`；旧 flag 仍保持显式，并以可见的 `legacy-cli` provenance 记录。
- `compatibility --repo <path>` 报告安装的 Runtime 与 attached repository schema 的
  `COMPATIBLE`、`MIGRATION_REQUIRED` 或 `INCOMPATIBLE`。`migrate plan` 是只读操作；
  `migrate apply` 没有 `--approved` 就拒绝写入，也不会重写 Work Item、evidence、decision、
  knowledge 或 archive history。
- 当 attached protocol files 完整存在时，有状态治理命令必须先得到 `COMPATIBLE`；
  `MIGRATION_REQUIRED` 和 `INCOMPATIBLE` 会在创建新 Work Item、生命周期 record、verification evidence、
  profile/adapter 写入或受治理 MCP 操作前 fail closed。迁移审查所需的只读诊断仍可用。

## Close 后的文档提升

结构化 `close` 后运行：

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item <id>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

第一条命令先验证准确 regular archive Contract 及其 raw digest、passing verification
receipt、唯一线性 finalization chain、sequence-2 `deleted` head、已合并 provider identity
与 approved close bindings，然后才更新受控文档字段。写入目标仅限 `status`、
`lastVerifiedBy`、四个 `terminal*` frontmatter 字段以及三语准确 parity 行。第二条命令是
必需的 quality/terminal-CI 模式，绝不写入。missing、foreign、ambiguous、malformed、
symlinked、mismatched 或 stale 输入都 fail closed。这些 repository helper commands
不表示 Runtime Core 会自动编辑文档。

## Contract/Summary 控制验证

repository library 提供 `validate_work_item_governance_controls`，供
Agent/MCP adapter 使用一个稳定报告同时检查 scenario coverage、acceptance
evidence、intent alignment 和可选的最终维度 receipt。验证器是只读的；缺失
字段只会报告 `blocked` 或 `unknown`，不会自动补全。最终 receipt 使用参考源
完整的 20 个维度；`fourPillarProjection` 是明确命名的可选展示视图，`4D`
不是协议字段。
当 adapter 提供当前 Runtime context 时，验证器还会要求 `runtimeVersion` 和
`runtimeDigest` 匹配；独立 value helper 只保证非空及带版本的 digest 格式。

## Runtime identity

`inspect`、`doctor`、MCP `initialize` 和 verification evidence 会提供 runtime version、runtime digest、
protocol version。`ai-cockpit --version` 只输出简短的 executable version。

## Release 验收边界

`tests/release/adopter_acceptance.sh` 是维护者侧的发布后 harness，不是 Runtime 命令。它下载并固定公开
Release binary，在隔离目录中执行 adopter lifecycle，并生成 `acceptance.json` 与 `SHA256SUMS`。不得用 workspace
build 或本地 target binary 替代；验收失败也不会改变已发布 Release truth。

其中的 lifecycle 必须完整执行：verification 之前先运行 `finalize-plan`，归档后必须通过
`finalize` 与 `finalize-verify`，并确认 head 为 `deleted`，然后才能用结构化决定执行 `close`。
fixture 可以使用显式的 retained resource receipt 作为中间 merge observation，再追加 deleted cleanup；
retained 不能授权新的 close。

验收 receipt 还会为每个隔离 root 保存带类型的 before/after manifest。`HOME` 与 `XDG_CONFIG_HOME` 的
`allowedPrefixes` 必须为空且保持不变；只有 `TMPDIR` 与 `CARGO_HOME` 允许 Runtime 写入，且 allowlist 明确限制为
`<TMPDIR>/**` 与 `<CARGO_HOME>/**`。清理状态记录在 `cleanup.json` 以及 `cleanupState`/`cleanupError` 中；清理失败
会使验收失败，但不会撤销或改写已发布 Release truth。

`tests/conformance/final_replacement_acceptance.sh` 是源码仓库的最终替代边界，记录安装的 Runtime identity、锁定的
reference oracle、conformance/adversarial/performance gate 和无复制检查，并生成 `acceptance.json` 与 `SHA256SUMS`。
