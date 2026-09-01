---
author: AI Cockpit maintainers
title: "参考源逐文件比较"
description: "按固定基线逐个比较参考源文件的分阶段方法。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# 参考源逐文件比较

本页说明 Rust 工程如何与维护者提供的本地参考源逐个文件比较。参考源是规格和行为语料，
不是应复制到 Rust Runtime 的目录。

## 固定基线

- 当前参考 checkout：通过 `AI_COCKPIT_REFERENCE_ROOT` 提供的本地 Git checkout；本轮比较固定为 `tests/conformance/reference-source.lock` 中的提交 `fde3380f81fea5fd2e288f7a8849f737dc074060`。
- Rust 比较基线：[xinglun/ai-cockpit](https://github.com/xinglun/ai-cockpit) 的 `origin/main`，提交 `cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`。
- 比较时使用的 Runtime：`ai-cockpit 0.2.47`，binary SHA256 为 `sha256:6b3bd6617c6372a17b1edf6f9dc9dbc016779146f67262265fd12d2a488bbc53`。

inventory 台账现在已显式重新绑定到本地 checkout。此前的
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 台账通过记录的 previous target revision 和 digest
仍可恢复，且不会被静默改写。当前 checkout 移除的路径记录在 `retiredReferencePaths`；源内容发生
变化但尚未重新逐文件审阅的非历史路径标为 `deferred-next-batch`，并保留之前的决定作为历史。
本地参考源缺失、dirty 或 commit 不匹配时必须 fail-closed。

为保持审计连续性，紧邻的上一比较基线仅作为历史记录保留：目标提交
`bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b`、Runtime `ai-cockpit 0.2.33`、
二进制摘要 `eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。

本页只报告当前固定的比较基线。历史交付细节保存在 Work Item 归档证据中，不放在面向读者的入口。

机器可读台账见
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json)。
回归检查要求每个参考源 tracked path 都有且只有一个分类，并拒绝首批未分类文件。
目标 checkout metadata 从固定 commit 派生，不受 dirty 或 untracked 工作树文件影响。

## 分类规则

- **implemented-equivalent**——相同的用户入口或治理责任已经存在，边界等效。
- **implemented-different-by-design**——责任存在，但由 Rust Protocol、共享外部 Runtime
  或显式 Agent adapter 以不同路径或抽象负责。
- **migrate-gap**——存在具体责任，但没有被接受的对应物，需要有界修复。
- **not-applicable**——超出当前 Runtime 产品边界。
- **reference-only**——只保留为解释或 conformance 资料，不是当前 Runtime 行为。
- **generated-history**——不可变参考历史或生成投影，绝不复制或静默改写。
- **deferred-next-batch**——已登记但语义比较安排在后续批次，不表示已经对齐或遗漏。

## 首批：治理入口

首批覆盖根目录 Agent 规则、`.ai` 入口和术语、面向读者的 README/架构入口，以及
参考源治理配置入口。Rust 工程保留重要边界，但不复制参考源的 Python Runtime、
Makefile target、YAML guard 树、provider 全局规则或生成历史。

| 参考表面 | Rust 结果 | 边界 |
| --- | --- | --- |
| `AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、Cursor rule | 有意采用不同实现 | 工程使用 attach 的 adapter 和显式 provider 安装。共享 Runtime 仍在工程外，不通过比较注入 provider 全局配置。 |
| `.ai/README.md`、glossary、cockpit workflow/adoption guide | 有意采用不同实现 | `.ai/README.md`、`.ai/glossary.md`、`docs/reference/agent-workflow.*` 和 getting-started 路线承载 Rust request-scoped Runtime 流程。 |
| 参考 guard、policy、quality、trust schema | 有意采用不同实现 | 对应控制由 typed Rust Protocol/Runtime 服务、repository tests、CI manifest 和参考文档提供；不复制源 YAML/JSON 文件。 |
| 根 README 与文档 README 入口 | 有意采用不同实现 | 三种语言入口互链，并说明共享 Runtime 与 repository context 隔离。 |
| `SECURITY.md` | 等效并增加 Rust 边界 | 保留安全策略入口，并补充 Runtime 部署与补丁边界。 |
| `CONTRIBUTING.md` | 本批补齐 | 现在说明显式 `--repo` 生命周期、fail-closed evidence、可见 Outcome、reviewed PR 和合并后的精确清理。 |
| 参考生成的 Work Item、decision、evidence、audit、release history | 生成历史 | 这些 bytes 作为参考历史保留，不复制到 Rust 工程。 |

因此，首批发现的唯一具体入口缺口（`CONTRIBUTING.md`）已补齐，但没有建立第二套
治理系统。其余文件已在台账中明确排入后续语义批次，不会被静默当作等效。

## WI-411：Java 多模块 fixture 边界

WI-411 逐一读取固定参考提交中的 `examples/fixtures/java-multimodule/`
九个文件，并全部标记为 `reference-only`。该 fixture 展示 Java 应用、模块间
依赖、本地 `javac`/`java` 检查以及临时升级/回滚；它是参考工程的可执行样例，
不是 AI Cockpit Runtime 代码，也不是可移植的企业证据。

| 参考路径 | 决定与目标边界 |
| --- | --- |
| `.gitignore` | 仅负责 fixture 临时构建产物；目标发布 harness 自己管理隔离临时目录。 |
| `app/src/main/java/fixture/app/Main.java` | Java 应用样例；目标只执行 adopter 明确声明的 argv，不随 Runtime 携带 Java 支持。 |
| `app/src/test/java/fixture/app/MainTest.java` | fixture 断言；目标 verification receipt 记录 adopter 命令，不复制该测试。 |
| `core/src/main/java/fixture/core/Decision.java` | 业务域样例策略；仓库策略必须显式类型化，不能从 fixture 复制。 |
| `core/src/test/java/fixture/core/DecisionTest.java` | 仅验证样例类，不是 Runtime 或企业证据。 |
| `evidence.json` | 源工程本地证据，包含 Maven/provider 不可用状态；目标发布 receipt 需要更严格的 identity 与 isolation 绑定。 |
| `fixture.json` | 源工程 stack/module 元数据；目标不从它推断 adopter capability。 |
| `pom.xml` | Maven 构建输入；Java/Maven 执行属于 adopter 或 delegated provider。 |
| `scripts/lifecycle.sh` | 源 fixture 编排脚本；目标生命周期由外部 Rust Runtime 和显式 repository-bound 命令提供。 |

不向 Rust 工程复制 Java 文件、Maven manifest 或源 shell 编排。未来第二技术栈
adopter 验收必须使用单独、明确授权的 Work Item；本批不宣称该能力。机器台账和
回归测试将九个路径绑定到上述决定，不能静默回退为 `deferred-next-batch`。

## WI-414：Python fixture 边界

WI-414 逐一读取固定参考提交中的 `examples/fixtures/python/` 四个文件，并全部标记为
`reference-only`。fixture 展示 Python service、打包元数据和 pytest 断言，但不是 Rust Runtime
代码、Python toolchain 支持承诺或可移植的企业证据。

| 参考路径 | 决定与目标边界 |
| --- | --- |
| `fixture.json` | 样例 stack、平台和路径元数据；目标保持对象工程事实本地化，不从本文件推断 Python capability。 |
| `pyproject.toml` | 样例打包与 pytest 配置；Python 安装和测试命令属于对象工程/Provider。 |
| `src/service.py` | 返回 `ok` 的应用样例；不是治理逻辑，不复制到目标工程。 |
| `tests/test_service.py` | fixture 专用 pytest 断言；不是 Runtime 或企业证据，对象工程必须声明自己的 verification 命令。 |

不向 Rust 工程复制 Python 源码、依赖清单、安装器或测试运行器。共享 Runtime attach 到 Python
对象工程后，仍提供相同的 Contract、evidence、lifecycle 与面向人的 Outcome 控制；但这只是
语义/文档对齐，不是 Python toolchain 或源命令兼容。机器台账与回归测试将这四个路径绑定到该
边界，防止它们静默回到 `deferred-next-batch`。

## WI-432：TypeScript web fixture 边界

WI-432 逐一读取固定参考提交中的 `examples/fixtures/typescript-web/` 十一个文件，并全部标记为
`reference-only`。fixture 展示 TypeScript 应用、npm 工具链、本地格式/ lint/测试以及样例生命周期，
但不是 Rust Runtime 代码、Node toolchain 支持承诺或可移植的 provider/企业证据。

| 参考路径 | 决定与目标边界 |
| --- | --- |
| `.gitignore` | fixture 本地构建产物清理；目标 release harness 自己管理隔离根目录。 |
| `evidence.json` | 源本地 npm evidence 与 provider 不可用声明；目标 receipt 需要显式命令和 identity 绑定。 |
| `fixture.json` | TypeScript/web stack 与路径元数据；Runtime 不从中推断对象工程 capability 或 Contract scope。 |
| `package-lock.json` | 对象工程拥有的 npm 依赖锁定文件；不是 Runtime 依赖或发布证明。 |
| `package.json` | 应用 build/test/lint/format/lifecycle 脚本；对象工程声明显式 argv，治理 lifecycle 仍由 Runtime 负责。 |
| `scripts/format-check.mjs` | fixture 专用格式规则，不是可移植治理控制。 |
| `scripts/lifecycle.mjs` | Node install/configure/block/upgrade/rollback/release 演练；不复制其 Runtime 治理和恢复语义。 |
| `scripts/lint.mjs` | 应用专用 lint 规则；对象工程负责自己的 lint 命令和 evidence。 |
| `src/index.ts` | 应用样例 evaluator；Runtime 不导入或推断其策略。 |
| `test/index.test.mjs` | 仅 fixture 的 Node 测试；对象工程必须声明并运行自己的 verification。 |
| `tsconfig.json` | 对象工程拥有的 strict TypeScript 编译配置；不承诺 Node/TypeScript toolchain。 |

不向 Rust 工程复制 TypeScript 源码、npm 依赖、安装器或 Node 生命周期脚本。attach 后的
TypeScript/web 对象工程继承共享 Contract、fail-closed evidence、repository isolation、lifecycle
和面向人的 Outcome 控制，但这是语义/文档对齐，不是 TypeScript toolchain 或源命令兼容。机器台账
与回归测试将十一个路径绑定到上述边界。

## WI-270：Contract 语义逐文件批次

WI-270 对下面 27 个参考源路径逐一检查。台账将它们标为
`implemented-different-by-design`：责任在 Rust Runtime 或仓库绑定的文档/测试中存在，
但不复制 Python 模块、Make 目标、生成文件或 provider 全局路径。对应项是证据索引，
不表示两边字节相同。

| 参考源路径 | 分类 | Rust 对应/边界决定 |
| --- | --- | --- |
| `docs/concepts/decision-states.ja.md` | 有意采用不同实现 | 日文 Contract/Outcome 文档与 typed decision 测试 |
| `docs/concepts/decision-states.md` | 有意采用不同实现 | Contract/Outcome 文档与 typed decision 测试 |
| `docs/concepts/decision-states.zh-CN.md` | 有意采用不同实现 | 中文 Contract/Outcome 文档与 typed decision 测试 |
| `docs/features/work-item-parallelism.ja.md` | 有意采用不同实现 | WI-123、日文配置路线、边界/lease 测试 |
| `docs/features/work-item-parallelism.md` | 有意采用不同实现 | WI-123、配置路线、边界/lease 测试 |
| `docs/features/work-item-parallelism.zh-CN.md` | 有意采用不同实现 | WI-123、中文配置路线、边界/lease 测试 |
| `docs/reference/safe-parallel-verification.md` | 有意采用不同实现 | Rust bounded executor、`verify --workers`、argv 与 evidence 测试 |
| `docs/reference/work-item-intelligence-interface.md` | 有意采用不同实现 | request-scoped status/intelligence 已有；完整 cost/wait/index-version 聚合仍是后续边界 |
| `docs/reference/work-item-state-machine.md` | 有意采用不同实现 | typed lifecycle/recovery/finalization；provider PR 状态属于外部 evidence |
| `docs/reference/work-item-status-interface.md` | 有意采用不同实现 | Rust status/Outcome projection 与测试替代 Python 生成 status 文件 |
| `scripts/ai_acceptance_policy.py` | 有意采用不同实现 | `governance_controls.rs` 校验 acceptance ID 与 evidence |
| `scripts/ai_check_scenario_coverage.py` | 有意采用不同实现 | Runtime scenario coverage 与 Contract/Summary 绑定 |
| `scripts/ai_check_work_item.py` | 有意采用不同实现 | typed Contract scope、authority、unknown、execution、concurrency、lifecycle 校验 |
| `scripts/ai_decision_protocol.py` | 有意采用不同实现 | 仓库绑定的 typed preflight decision receipt |
| `scripts/ai_intent_policy.py` | 有意采用不同实现 | Runtime intent alignment 与 intent/scenario binding |
| `scripts/ai_parallel_verification.py` | 有意采用不同实现 | Rust bounded execution、worker cap、确定性结果与 scope safety |
| `scripts/ai_preflight_review.py` | 有意采用不同实现 | typed preflight state、humanDecisionRequest、确认与恢复条件 |
| `scripts/ai_scenario_policy.py` | 有意采用不同实现 | 风险敏感 scenario policy 与 fail-closed unknown |
| `scripts/ai_work_item_state.py` | 有意采用不同实现 | Rust lifecycle state machine 与 recovery receipt |
| `tests/test_acceptance_policy.py` | 有意采用不同实现 | Rust Contract schema/preflight regression |
| `tests/test_ai_parallel_verification.py` | 有意采用不同实现 | Rust CLI/executor verification regression |
| `tests/test_checkpoint_intent.py` | 有意采用不同实现 | Rust preflight/checkpoint intent regression |
| `tests/test_contract_and_policy.py` | 有意采用不同实现 | Rust strict Contract/policy regression |
| `tests/test_intent_policy.py` | 有意采用不同实现 | Rust intent alignment regression |
| `tests/test_parallel_lifecycle_contract.py` | 有意采用不同实现 | Rust parallel boundary、lease、lifecycle、isolation regression |
| `tests/test_preflight_review.py` | 有意采用不同实现 | Rust preflight/review regression |
| `tests/test_scenario_coverage_gate.py` | 有意采用不同实现 | Rust required-scenario 与 invalid-status regression |

本批未发现未记录的 Contract 语义实现缺口。intelligence-interface 行保持有界说明：
request-scoped status 和 evidence-derived Outcome 已实现，参考源更广的聚合与 cost/wait
维度仍排入后续批次，不能当作完整 parity。

## 当前台账快照

<!-- reference-inventory-counts: total=4450 generated-history=3681 implemented-different-by-design=259 implemented-equivalent=1 not-applicable=4 reference-only=62 deferred-next-batch=443 migrate-gap=0 -->

下面的机器校验表是当前快照的唯一来源；三个语言页面使用相同的规范 key。
当前参考源集合有 4,450 条路径。追加式台账共有 5,119 条记录，因为它保留了上一参考基线
中已移除的 669 条路径。deferred 记录仍是待比较工作，不是 parity 声明。本次重绑定还记录了
160 条当前源内容发生变化的路径，capability/profile slice 已没有 `migrate-gap`：

| 指标 | 数量 |
| --- | ---: |
| `current-tracked-paths` | 4,450 |
| `generated-history` | 3,681 |
| `implemented-different-by-design` | 259 |
| `implemented-equivalent` | 1 |
| `not-applicable` | 4 |
| `reference-only` | 62 |
| `deferred-next-batch` | 443 |
| `migrate-gap` | 0 |
| `retired-reference-paths` | 669 |
| `append-only-ledger-records` | 5,119 |

1. `.ai/project/adopter-capability-manifest.json` 已从当前本地 checkout 移除；其旧决定保留在
   `retiredReferencePaths` 中，仍是 installer-surface 外部边界，而不是当前记录。
2. `.ai/project/capabilities.json` 由严格 Rust-native declaration 与显式 operation mapping 表达。
3. `.ai/project/success_criteria.json` 作为不具授权能力、绑定 snapshot 的可见 projection 表达。
4. `.ai/project_profile.yaml` 由 `.ai/project.json` 与严格 JSON `profile-policy.json` projection 表达。

治理入口、getting-started 路线、CI/release 边界与 capability/profile projection 在源内容未变化时
保留之前有证据的决定。已变化路径明确延期，必须在后续逐文件批次中重新阅读本地源；已移除路径仅
作为历史记录。以上三条当前记录是有界的 Rust-native counterpart；第四条仅是历史路径。

## WI-435：本地参考源重新绑定

WI-435 将当前台账绑定到维护者提供的本地 checkout 提交
`fde3380f81fea5fd2e288f7a8849f737dc074060`，但不把“源更新”冒充成“语义比较”。当前 manifest
包含完整 tracked path 集合，并记录 previous source commit、previous manifest digest、changed
paths 和 retired paths。发生变化的非历史记录会保持 `deferred-next-batch`，直到后续 Work Item
重新逐个读取；被移除的路径保留为历史元数据，不会成为看不见的遗漏。这样既不依赖公开参考仓库，
又保持本地源更新前后的审计连续性。

WI-274 只将目标 checkout metadata 和 canonical comparison snapshot 重新绑定到已审阅的
默认分支提交。WI-273 保持为不可变的失败交付记录：其首次提交无法证明 parity 登记先于
verification evidence，因此 successor 会隔离保留这段历史，不会改写它。

## WI-437：本地参考源治理规则重新比对

WI-437 重新逐个阅读了此前公共参考台账与维护者本地 checkout 之间发生变化的 7 个治理文件。
7 个文件全部判定为 `implemented-different-by-design`：没有发现 Rust Runtime 功能遗漏，也不需要
复制源工程产物。

| 本地参考路径 | Rust 结果 | 文件级决定 |
| --- | --- | --- |
| `.ai/cockpit/README.md` | 有意采用不同实现 | 源工程删除了 Python 模板的 Implementation Approach 章节；Rust 仍在 typed Runtime/文档中保留有 evidence 绑定的 approach 与 Outcome projection。 |
| `.ai/cockpit/README.ja.md` | 有意采用不同实现 | 源工程删除了过时的 `REPORT_LANGUAGE` Make 参数；Runtime 自己的 presentation 本地化已覆盖该边界。 |
| `.ai/cockpit/adoption.ja.md` | 有意采用不同实现 | 源工程 onboarding 不再传 `REPORT_LANGUAGE`；Rust onboarding 没有模板本地 Make 命令，而是使用显式 `--repo`。 |
| `.ai/guards/changed_critical_coverage_policy.json` | 有意采用不同实现 | 被删除的 Python 专用 coverage 关联由 native tests、governance integrity 和 typed Runtime controls 承担。 |
| `.ai/guards/coverage_policy.yaml` | 有意采用不同实现 | 源工程关联注册表不是 Rust 配置面；coverage ownership 由 native tests 和 CI gate manifest 表达。 |
| `.ai/quality/governance-routing.yaml` | 有意采用不同实现 | 源工程将 route 选择与重复的 depth/evidence 字段分离；Rust 通过 dynamic routing 与版本化 gate manifest 保持相同分离。 |
| `.ai/schemas/task_outcome.schema.json` | 有意采用不同实现 | 源工程简化了 Python Task Outcome schema；Rust `OutcomeV2`/`humanHandoff` 是独立 typed Protocol/presentation contract，不因源 schema 删除而移除，也不复制源 schema。 |

因此，本次源 diff 是 Python/Make 表面的参考侧清理，不是可移植的功能增量。台账仍保留
`previousBatch`、`previousClassification`、`sourceChangedSincePrevious` 等源变更溯源信息，同时
7 个当前记录不再延期。这样可以避免同一批本地源文件在后续比对中反复打开。

## WI-441：本地参考源入口与 Agent 语义对齐

WI-441 在维护者本地参考源提交 `fde3380f81fea5fd2e288f7a8849f737dc074060` 上，逐个重新阅读 9 个入口和
Agent-facing 文件。参考 checkout 为 `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`；不访问公开仓库，
托管 CI 只使用已提交的离线语料。

| 本地参考路径 | 分类 | Rust 对应物/有界决定 |
| --- | --- | --- |
| `AGENTS.md` | implemented-different-by-design | `AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.md` 和 typed lifecycle/adapter 服务保留 Contract-first、最新 base、人工暂停、关闭和清理；源 `make ai-*` 仍只属于源工程。 |
| `GEMINI.md` | implemented-different-by-design | `crates/cockpit-agent` 生成的显式 Gemini adapter 投影可迁移的 Contract/Summary/checkpoint 语义；不复制 provider 专用根文件或全局配置。 |
| `docs/README.md` | implemented-different-by-design | `docs/README.md` 与 current/getting-started/operations/reference 路线保留源 reader-first、goal-first 意图，并明确 Rust 边界。 |
| `docs/README.zh-CN.md` | implemented-different-by-design | 中文读者路线保留相同意图和语言互链，并明确 Runtime/adopter 所有权。 |
| `docs/README.ja.md` | implemented-different-by-design | 日文读者路线保留相同意图和语言互链，并明确 Runtime/adopter 所有权。 |
| `docs/capabilities.md` | implemented-different-by-design | 目标保留 Repository Governance Layer 与外部非声明，并提供 Rust CLI/MCP、scaffold、profile、knowledge、Outcome 和隔离路径。 |
| `docs/capabilities.zh-CN.md` | implemented-different-by-design | 中文能力路线保留源边界，说明 repository-local Runtime/adopter 继承，不复制源状态字节。 |
| `docs/capabilities.ja.md` | implemented-different-by-design | 日文能力路线保留源边界，说明 repository-local Runtime/adopter 继承，不复制源状态字节。 |
| `docs/features/task-outcome-report.md` | implemented-different-by-design | `OutcomeV2`、CLI/MCP 面向人的 handoff 与不可变 evidence 保留 report/status/PR 分离；源文字和 Make 命令不是 wire 要求。 |

9 个记录均已逐个分类，不再是 deferred。这里是语义对齐，不是源文件或 JSON wire 对齐：Rust repository 没有
`GEMINI.md` 并不构成全局 provider 遗漏，因为 `agent install --provider gemini` 是显式、可回滚且绑定 repository 的操作。
对象工程同样继承一份共享 Runtime 与隔离的 `.ai/` 边界。

## WI-461：getting-started onboarding 重新基线

WI-461 重新阅读历史比较提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 到维护者本地参考源固定提交
`fde3380f81fea5fd2e288f7a8849f737dc074060` 之间发生变更的九个入门页面。参考 checkout 为
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`；不访问公开参考仓库，也不复制源实现。

| 固定参考路径 | 分类 | Rust 对应物/有界决定 |
| --- | --- | --- |
| `docs/getting-started/first-work-item.md` | implemented-different-by-design | Rust 三语 first-Work-Item 页面保留 repository-bound 全生命周期、可见 Outcome、人工 review 停止点和精确清理，并使用原生 CLI；不复制源 Make 命令及已删除的 `REPORT_LANGUAGE` 参数。 |
| `docs/getting-started/first-work-item.zh-CN.md` | implemented-different-by-design | 中文页面保留相同生命周期和显式 `--repo` 边界；展示语言不会改变 Contract 事实。 |
| `docs/getting-started/first-work-item.ja.md` | implemented-different-by-design | 日文页面保留相同生命周期与 provider-resource 边界，本批修正重复的 merge 段落。 |
| `docs/getting-started/security-release-verification.md` | implemented-different-by-design | Rust release/distribution 与 installation-security 页面通过当前 manifest/SHA256SUMS 路径保留 tag、digest、SBOM、provenance、provider 责任和 adopter 隔离语义；不复制源 `release.json` 投影。 |
| `docs/getting-started/security-release-verification.zh-CN.md` | implemented-different-by-design | 中文发布路线使用 Rust 原生资产与外部 provider 边界，保留证据分离和不一致时 fail-closed 处理。 |
| `docs/getting-started/security-release-verification.ja.md` | implemented-different-by-design | 日文发布路线保留 digest、provenance、SBOM 和公开 adopter 限制，不导入源安装器行为。 |
| `docs/getting-started/standard-adoption-guide.md` | implemented-different-by-design | Rust 指南保留 reader-first 的 install、attach、calibration、adapter、Work Item、Outcome、merge、cleanup、close 阶段，并使用共享 repository-bound Runtime。 |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | implemented-different-by-design | 中文指南保留有序 adoption 边界和显式仓库所有权，并使用 Rust CLI 路径。 |
| `docs/getting-started/standard-adoption-guide.ja.md` | implemented-different-by-design | 日文指南保留有序 adoption 路径与共享 Runtime 边界，不复制源专用命令。 |

9 条记录现已逐一完成决定。本批是语义/文档对等，不是源文件或 JSON wire 对等。台账继续保留
`sourceChangedSincePrevious`、`previousBatch`、`previousClassification` 比对溯源，同时移除本批记录的 deferred 状态。

## WI-464：工作流与构建重新基线

WI-464 在维护者本地参考源固定提交
`fde3380f81fea5fd2e288f7a8849f737dc074060` 上，重新阅读此前工作流比对后发生变化的四个源路径。
不复制源实现。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | 源 ShellCheck 安装和 Python/多技术栈矩阵仍属于源/provider 边界。Rust 保留自身审核过的 action-runtime policy、动态质量路由、Rust workspace/平台检查和公开 adopter 验收。 |
| `.github/workflows/release.yml` | implemented-different-by-design | 源 `release-digests.json` 归档投影及删除旧 `release.json` 双资产检查，对应 Rust release manifest/`SHA256SUMS`、SBOM/provenance、平台 smoke 和 adopter 证据；不复制源投影 bytes。 |
| `.github/workflows/smoke.yml` | implemented-different-by-design | 源文件移除了 `REPORT_LANGUAGE` Make 参数。Rust 没有源 `smoke.yml`；CI、release、gate manifest 和不可变 adopter harness 通过显式仓库上下文承担有界检查。 |
| `Makefile` | implemented-different-by-design | 源 Python/Make 分片、knowledge 和语言辅助逻辑仅属于源工程。Rust 使用 Cargo、CLI、规范 gate manifest 和显式 `--repo`，不需要第二套 Make 治理层。 |

目标工程的 action pin 继续由自身审核过的 action-runtime policy 管理，不会把源矩阵 pin 静默替换到 Rust 路径。台账解决这四个源变更记录，同时保留
`sourceChangedSincePrevious` 溯源；没有发现 Rust 遗漏。对象/采用方工程继承共享 Runtime 与隔离的仓库证据边界，而不是源工作流文件。本批是语义/文档对等，不是源文件、provider、Python/Make 或 JSON wire 兼容。

## 批次顺序

后续批次按以下顺序比较并在必要时实现有界差异：

1. Contract 字段、intent、scenario/acceptance 维度、并行 slot 与 preflight review。
2. CI quality routing、动态 verification tier 与 evidence assurance。
3. Runtime lifecycle、Outcome/MCP projection、recovery、knowledge 与 repository isolation。
4. Conformance、荒诞/对抗场景、性能、发布与 adopter 验收。

每批都有独立 Contract 和 evidence。批次通过 review 并发布后，下一批使用已发布 Runtime
重新验收，避免工作树代码伪装成发布行为。

## WI-286——Agent Risk 与 checkpoint 文件级批次

WI-286 逐个比较参考源 Agent Risk/checkpoint 责任。源 Python/YAML 仍只作为参考
corpus；Rust typed Protocol record 与共享 lifecycle validator 执行有界语义。

| 参考路径 | 分类 | Rust 对应 |
| --- | --- | --- |
| `.ai/guards/agent_risk_policy.yaml` | implemented-different-by-design | typed `checkpointPolicy`、Contract verification 声明、Agent Risk validator 与动态 profile 文档。 |
| `scripts/ai_check_agent_risk.py` | implemented-different-by-design | `validate_agent_risk_controls` 在 lifecycle 边界复用。 |
| `scripts/ai_checkpoint.py` | implemented-different-by-design | typed `CheckpointEvidence`、amendment CLI、append-only chain 与 resume-stale 绑定。 |
| `tests/test_ai_agent_risk.py`、`tests/test_ai_checkpoint.py`、`tests/test_outcome_lifecycle_rules.py` | implemented-different-by-design | Rust protocol/repository lifecycle 与 Agent 规则静态 parity test。 |

本批次是 semantic parity，不是直接 JSON-wire parity。WI-291 已加入 read-only
Rust Contract-aware CI gate，并在收敛阶段保留 Python route/manifest 作为 shadow；完整
workflow 与 release-preflight parity 仍然 deferred。

## WI-287 checkpoint 一致性收敛

WI-287 关闭了 checkpoint 实现与测试源文件仍处于 deferred 的两条台账记录。
Rust 现在明确拒绝 verification 已开始后的 `before_edit` checkpoint，并拒绝
无效的最新 resume timestamp。参考测试语义由 Rust-native lifecycle regression
表达，不复制 Python 测试或 source wire shape。静态 Agent-rule test 也用本项目
规则断言相同的终态与窄 successor 边界。

对象工程边界不变：共享 Runtime 仍是 request-scoped，每个操作显式带 `--repo`，
human Outcome 是可见交付边界。CI workflow 收敛和更广的 adopter surface 仍是独立
有界批次。

## WI-291——CI Contract 感知质量门

WI-291 比对参考源 workflow quality routing 与 preflight 边界，并将其接入 Rust-native
CI surface。Python route 继续作为 `light`/`standard`/`strict` 的动态 planner，规范
manifest 继续作为命令清单。在 standard/strict Pull Request 执行仓库命令前，Rust CLI
只读 `gate` 校验 active Contract、repository/base/snapshot identity、
intent/scenario/operation/stage route 和 Agent-Risk/preflight 投影。它输出带 identity
绑定的 `repository_contract_quality_gate` receipt；黄色或红色以 fail-closed 方式阻止 CI。
该 gate 不写入 `.ai/` 记录。

本批次是 semantic parity，不是复制源 YAML 或 Python wire。CI 源码构建的 Runtime identity
仅供诊断；不可变 Release/adopter identity 仍由发布 artifact 验收边界负责。参考源剩余
workflow 矩阵、gate metadata/timeout、release preflight 和多技术栈 adopter 仍在 ledger
中 deferred，不能宣称已实现。

## WI-302 首批 deferred 文件比对

WI-302 按字典序将前 10 个 deferred 路径与固定参考源提交逐文件比对。其中 8 条记录已经
得到有证据支持的结论。WI-304 随后比对了包含参考源广泛 Python/多技术栈矩阵的两个
workflow，并记录 Rust-native 拆分与对象工程/外部 adopter 边界。

| 参考源路径 | 分类 | Rust 对应/边界 |
| --- | --- | --- |
| `.ai/cockpit/bandit_low_risk_baseline.json` | 不适用 | 这是参考源 Python 工具的 Bandit 生成基线，Rust Runtime 没有 Python/Bandit 产品表面。 |
| `.gitattributes` | 有意采用不同实现 | Rust source archive 边界及 `tests/release/source_archive_policy_test.sh` 排除治理/构建目录，同时保留 Cargo 源码。 |
| `.github/CODEOWNERS` | 不适用 | 参考源的个人 owner 不可移植；adopter 的 review owner 由外部 repository/provider 决定，并在 contributor/adopter 文档中说明。 |
| `.github/dependabot.yml` | 不适用 | pip/Actions 更新是 provider 可选自动化；Rust 依赖事实由 `Cargo.toml`/`Cargo.lock` 与 action pin policy 表达。 |
| `.github/workflows/compatibility.yml` | 有意采用不同实现 | WI-304 比对了 ShellCheck、lockfile、Python、real/extended/mobile 矩阵及非阻断 latest probe。Rust `ci.yml`、动态质量路由、规范 gate 与公开 adopter 验收负责 Rust 产品；参考源 installer/Python/多技术栈覆盖明确属于 adopter/外部边界。 |
| `.github/workflows/release.yml` | 有意采用不同实现 | Rust release workflow 与 release tests 提供目标 archive、checksum、SBOM/provenance、平台 smoke 及公开/N-1 adopter 验收。 |
| `.github/workflows/smoke.yml` | 有意采用不同实现 | WI-304 比对所有 source shard、dispatch input、artifact、依赖边、release/measurement 条件和 installer 检查。Rust `ci.yml`、`release.yml`、gate manifest 与不可变 adopter harness 拆分承担这些责任；参考源 Python/Make/install smoke 明确由外部/adopter 负责。 |
| `.gitignore` | 有意采用不同实现 | Rust/Cargo 构建与治理 review 路径被忽略，source archive policy 有回归测试。 |
| `LICENSE` | 有意采用不同实现 | 两边都是 MIT；版权主体和 Rust packaging 按目标工程定义，不复制参考源文本。 |
| `Makefile` | 有意采用不同实现 | Rust CLI、Cargo 和显式 CI/release 脚本替代 Python Make 编排，并保持 request-scoped `--repo`。 |

WI-302/WI-304 批次完成时的台账快照为：4,262 条
`generated-history`、190 条 `implemented-different-by-design`、1 条
`implemented-equivalent`、3 条 `not-applicable`、3 条 `reference-only`、660 条 `deferred-next-batch`。
两个 workflow 已作为 Rust-native 的有意差异边界关闭；这不表示参考源 Python installer
或多技术栈矩阵会在 Rust Runtime 内运行。

## WI-304 workflow 比对

WI-304 在固定参考源提交上比对 `.github/workflows/compatibility.yml` 与
`.github/workflows/smoke.yml`，覆盖 trigger、permission、concurrency、每个 job 与 matrix、
`needs` 依赖、dispatch input、artifact 上传/下载、阻断与非阻断条件、release/measurement
分支及 installer 检查。

`compatibility.yml` 有八类责任：对参考源 `install.sh` 的 ShellCheck；固定 Python 平台与
lockfile 可复现性；real、extended、mobile 技术栈质量矩阵；非阻断的 latest ecosystem
probe；以及独立的阻断/latest 汇总 gate。Rust 对应责任刻意拆分：`ci.yml` 选择仓库动态
`light`/`standard`/`strict` 路由和规范 gate manifest，Rust workspace/platform 检查及公开
adopter harness 验证 Runtime 与 Repository Protocol。目标没有 `install.sh`、Python lockfile
或参考源 Make 编排，因此 adopter 的工具链/技术栈覆盖由 adopter 或其 hosted provider
配置并提供证据，不静默宣称为产品 parity。

`smoke.yml` 有 project-test manifest/core/governance/installer/lifecycle/release 分片、模板
汇总、installation smoke、条件式 release evidence 以及最终 CI evidence receipt。Rust 目标将
对应边界放在 `ci.yml`（Contract-aware quality、Windows、锁定的行为 oracle）、`release.yml`
（archive、SBOM、checksum、provenance、release policy）、规范 gate manifest 和严格的公开/
N-1 adopter 验收中。参考源 Python test shard、`install.sh`/Make smoke 与探索性的 latest
toolchain probe 没有目标等价物，明确属于外部或 adopter 责任。

这是责任语义 parity，不是 workflow 字节或 source command parity。目标 Shell 脚本目前有
语法校验；针对目标脚本增加 ShellCheck 是独立的 CI hygiene 决策，因为参考源 gate 检查的是
目标不存在的 installer。本批不复制任何参考源 Python module、Make target、installer 或多
技术栈 fixture。

## WI-305——架构、安装与验证文件级批次

WI-305 在固定提交上逐个比对接下来的四个 deferred 参考文件。它们分别描述只读安装侦测器、
可选的十阶段交互式 Installer Wizard、按阶段的轻量验证，以及 Wizard 输入/本地化原语。目标
不是复制这些 Python adapter 的字节，而是逐项记录 Rust Runtime、adopter 外部边界或
reference-only 结论。

| 参考路径 | 分类 | Rust 对应物 / 有界结论 |
| --- | --- | --- |
| `docs/architecture/installation-detection-boundary.md` | implemented-different-by-design | `inspect`、`status`、`doctor`、`attach`、`profile propose`、首次校准文档和 CLI attach/profile 测试提供只读事实与显式写入边界。不可变 Release 安装与 repository onboarding 分离。 |
| `docs/architecture/interactive-installation-wizard.md` | reference-only | 源十阶段 wizard、dry-run Installer preview 与确认界面不属于 Rust Runtime。目标支持公开 Release 校验后显式执行 `inspect` → `attach` → profile review/confirm → `doctor`；Agent adapter 可提供对话 UI，但不能制造批准。 |
| `docs/architecture/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | typed stage、policy 驱动 tier、fail-closed 治理决定、显式 skipped/unknown、单次 request-scoped context、动态 `light`/`standard`/`strict` 路由和 advisory cost/reuse telemetry 由 verification route、CI gate 和 cost 测试覆盖。源 `hard`/`soft`/`informational` checker 标签作为文档边界保留，不复制成通用 wire enum；源 Make/Python checker 编排不复制。 |
| `docs/architecture/wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP 人类 Outcome 和命令展示支持 `en`/`zh-CN`/`ja`，保留 Contract 值原文，并在显式命令/preflight 边界 fail closed。由于目标没有交互式 Installer Wizard，不提供 Wizard 专用 TTY back/pause/help；对话控制由 adapter 负责。 |

### 文件级发现与迁移边界

源 detector 的 `new_adoption`/`upgrade` 区分，在目标中对应 Release 安装与 repository-local
attach/profile 决定的分离。目标 inspection 是只读的；`attach` 和 profile confirm 是显式
repository 写入，任何命令都不会从 prose 或检测到的技术栈推导 authority。active Work Item、
dirty state、conflict、symlink risk 和缺失事实仍然是停止或请求审查的理由，而不是猜测的依据。

源 interactive wizard 是其 Python Installer 外层的便利层，不是把 Installer 安装进 Rust
repository 的要求。它的十个阶段、dry-run、取消、rollback boundary 与不 commit/push/PR/merge
的承诺，在目标安装路线中作为 adopter 边界说明；目标不提供第二套 transaction authority，
也没有可以绕过 Contract/preflight/human decision 的交互提示。

源 soft-gate 文档的 `hard`、`soft`、`informational` 区分不会复制成目标通用 wire enum，而是
映射为 fail-closed 治理决定和显式 advisory observation 的安全边界。不适用某阶段的检查也会
显式输出，而不是被省略；trend 与 cost observation 仍是 advisory；`pre_ci` 不能变成 hosted CI
evidence。tier 和 assurance 由 policy 绑定，不由执行速度推断。
这同样适用于对象工程：共享 Runtime 在外部、每个请求显式带 `--repo`，provider/enterprise
控制属于 delegated evidence。

本地化只作用于 presentation。Runtime 生成的标题、状态、unknown、恢复文字和下一步可以按
配置语言显示；路径、命令、Contract intent、acceptance criteria 与 machine evidence 保持
编写时的值。这不代表 Runtime 提供通用翻译或 source-compatible Wizard UI。此批未发现
`migrate-gap`；交互式 wizard 仍是明确的 reference-only 边界，而不是未记录的遗漏。

## WI-308：证据治理、信任层与回滚腐化案例

WI-308 在固定参考提交 `e5acb677` 上逐个比对四个文件：一个演示用视觉资产、假设性的
回滚腐化案例、Evidence Governance 和 Trust Layer。目标逐条记录结论，不复制参考源实现
或二进制资产。

| 参考源路径 | 分类 | Rust 对应/有界结论 |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | 固定 GIF 为 GIF89a、800x435、587,945 字节，SHA-256 为 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795`。仅作为视觉参考，不复制二进制，也不宣称它是 Runtime 合同。 |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | 三语 adversarial-validation 文档与 typed Contract/scope 校验覆盖越权路径、无关变更和受控恢复。案例仍是假设性的；Runtime 不自动回滚、不批准合并，也不推断业务影响。 |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | `docs/security/enterprise-governance.*`、`docs/reference/outcome-report.md` 以及 typed Protocol/Repository evidence 投影 Evidence → Governance Decision → Human Control 链。Provider evidence 仍由外部负责，文字本身不是证明。 |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | `docs/architecture/product-boundary.md`、`docs/philosophy.md`、企业治理文档和 Runtime capability truth registry 定义校准信任、fail-closed 未知、人类控制及明确非目标；源 public claim matrix 不是目标 gate。 |

这里是语义责任 parity，不是 source wire 或字节兼容。目标更严格的 Contract/evidence schema 与共享
request-scoped Runtime 保留源安全意图，并增加 repository identity、snapshot、人类决定和 provider
边界。GIF 明确为 reference-only；不复制 Python、Make、installer 或二进制，也不把本地 evidence
提升为 provider/enterprise assurance。日文文档提供同样结论和阅读路径。

## WI-323 参考文档基础

WI-323 在固定参考源提交上逐个比对接下来的九个 deferred 文档路径。本批只关闭文档责任，
不复制源工具，也不改变 Runtime authority。

| 参考源路径 | 分类 | Rust 对应/有界结论 |
| --- | --- | --- |
| `docs/contributing/installation-document-maintenance.md` | implemented-different-by-design | 三语参考路线与文档 acceptance 脚本保留精简首页、链接/元数据、版本中立、no-guess/no-overwrite/no-fallback 和单独批准边界。 |
| `docs/current/README.md` | implemented-different-by-design | `docs/current/README.*`、`.ai/README.md`、`.ai/glossary.md`、`AGENTS.md` 与 `docs/reference/README.*` 构成当前 Agent 读取路线。源 `make ai-documentation-read-set` 不是目标命令。 |
| `docs/design/harden-work-item-pr-closure.md` | implemented-different-by-design | `docs/reference/agent-workflow.*`、`docs/reference/commands.md` 与 Rust lifecycle 强制最新 base、专用分支、reviewed PR、先合并后关闭、同步和精确清理；provider PR 操作仍属外部。 |
| `docs/distribution.md` | implemented-different-by-design | 目标当前路线与 `docs/release/distribution.*` 提供兼容入口、不可变 artifact 安装和发布后 adopter 边界。 |
| `docs/enterprise-security-boundary.md` | implemented-different-by-design | `docs/security/enterprise-deployment-boundary.*`、`enterprise-governance.*` 与 `SECURITY.md` 将 repository evidence 与 delegated identity、sandbox、audit、认证控制分开。 |
| `docs/examples/trust-layer-demo.sh` | reference-only | 离线 stop/continue 示例保留为参考说明；目标证据是 typed Runtime preflight、capability、intent 和 adversarial 测试，不复制 shell authority。 |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | Rust `OutcomeV2`、`work-item outcome`、MCP `work_item_outcome` 与三语 handoff 测试保留面向人的报告顺序及 evidence 边界。 |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | 中文展示沿用同一 Rust Outcome/MCP 路由；Contract 验收原文保持 authored value，不自动翻译。 |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | 日文展示沿用同一 Rust Outcome/MCP 路由；Contract 验收原文保持 authored value，不自动翻译。 |

对 Cursor adopter 反馈按版本归一化后，当前 Runtime 已提供稳定 stdout JSON 和人类 handoff，
`work-item new`/`start` 会阻断未关闭 archive 与事前已有变更，readiness 也有显式结果。
CLI 无法强制 Cursor 展开聊天面板；provider/Agent adapter 必须展示或重放 human handoff。
诊断修复、close-gap 便利命令和可选 controls 脚手架属于后续产品决策，本批不把它们静默
宣称为参考源 parity。目标也不要求 `Makefile.ai`；显式 `--repo` 的 CLI/MCP 是 repository
中立的 adopter 接口。

本批是语义责任 parity，不是 source wire 或字节 parity。源 Make/Python 报告生成器、installer
脚本和 trust demo 不复制。对象工程边界与所有 adopter 一致：一份共享外部 Runtime、每个
repository 独立的 `.ai/` 状态、显式 repository context，以及由 provider 负责对话展示。

## WI-326：质量门、总览、设计思想与关闭计划文件级批次

WI-326 逐一比对固定参考源中的以下 9 个路径。其中 8 个按有意不同的实现登记；关闭强化计划
保留为 reference-only，因为它是内部历史计划，不是当前 Runtime 命令契约。

| 参考源路径 | 分类 | Rust 对应/有界结论 |
| --- | --- | --- |
| `docs/non-make-adaptation.ja.md` | implemented-different-by-design | 安装与 Agent workflow 路线表达外部 Runtime 和仓库本地 adapter 边界。对象工程自有的技术栈命令仍在 Core 之外；不复制也不要求源 `Makefile.ai` 桥接层。 |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | 日文 CI 质量门与 manifest 路线保留门禁所有权、证据、追踪以及按策略选择 `light`/`standard`/`strict` 的动态路由。不复制源 Make 目标、Python checker 注册表或模板维护 fixture。 |
| `docs/operations/quality-gates.md` | implemented-different-by-design | 版本化 Rust 原生门禁清单与 CI 路由保留源质量门语义，同时让托管 CI 与对象工程技术栈检查各归其责任边界。 |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | 中文质量门与 manifest 路线保留同样的证据和动态路由边界；源 Make/Python 编排不是目标命令。 |
| `docs/overview.ja.md` | implemented-different-by-design | Rust architecture、capabilities、Agent workflow 与 command 路线保留源五层总览，并以 request-scoped、repository-bound 方式治理；不复制源 status/verification registry。 |
| `docs/philosophy/design-philosophy.ja.md` | implemented-different-by-design | 日文产品边界、能力和企业治理文档保留校准信任、证据优先于自我声明、与风险相称的控制以及人的责任。 |
| `docs/philosophy/design-philosophy.md` | implemented-different-by-design | 英文产品边界、能力和企业治理文档保留同样原则；Core 不是 Agent Runtime、安全沙箱、身份提供方或合规证书。 |
| `docs/philosophy/design-philosophy.zh-CN.md` | implemented-different-by-design | 中文产品边界、能力和企业治理文档保留同样原则与明确非目标。 |
| `docs/plans/harden-work-item-pr-closure.md` | reference-only | 源文件是 Python `ai-finish`/`ai-close` 的内部历史强化计划。当前 Rust lifecycle 与 governance-integrity 路线保留关闭意图，但过时的实现步骤和命令名不是当前能力。 |

本批没有 `migrate-gap`。这里是语义边界对应，不是 source wire 或字节兼容：质量决定由版本化
manifest 与当前 Runtime 负责，托管 provider 检查、对象工程技术栈命令和企业控制仍由各自责任方
提供。动态路由由策略选择；不会只因执行速度就推断更严格 tier，也不会把 tier 当作 assurance。
对象工程使用发布版 Runtime 时同样必须显式携带 `--repo`。

## WI-327 采用方、校准与长周期文档切片

WI-327 在固定参考源提交上逐个比较接下来的九个 deferred 路径。其中八条是有意采用不同
实现，Bandit 扫描审计是源 Python 工具链特有的历史记录，因此保留为 reference-only。

| 参考源路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | 有意采用不同实现 | 发布二进制采用方与升级验收脚本、分发路线及日文生命周期/安全文档保留隔离安装、生命周期、回滚和清理证据。不复制源多技术栈 fixture 与 Make/Python 编排。 |
| `docs/reference/adopter-long-cycle-validation.md` | 有意采用不同实现 | 发布二进制采用方与升级验收脚本、分发路线及生命周期/安全文档保留隔离安装、生命周期、回滚和清理证据。不复制源多技术栈 fixture 与 Make/Python 编排。 |
| `docs/reference/adoption-reality-report.md` | 有意采用不同实现 | Runtime capability/profile/status projection 与不可变 adopter 验收 receipt 区分模板能力、采用方执行、provider evidence 和企业 assurance；不会把本地文件提升为外部证明。 |
| `docs/reference/bandit-synchronization-security-audit.md` | 仅供参考 | 这是源 Python 工具链的 Bandit 历史发现清单。目标没有 Python/Bandit 产品表面，也不宣称源发现数量或 digest；Rust 原生质量门和 threat model 边界单独维护。 |
| `docs/reference/calibration-inventory.md` | 有意采用不同实现 | 仓库绑定的 profile proposal/confirm、capability/status projection 与显式 unknown 保留事实/证据边界，不复制源十列 Python inventory。 |
| `docs/reference/calibration-profiles.ja.md` | 有意采用不同实现 | 日文校准指南和严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级与明确降级证据；校准与单个 Work Item 质量路由分离。 |
| `docs/reference/calibration-profiles.md` | 有意采用不同实现 | 校准指南和严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级与明确降级证据；校准与单个 Work Item 质量路由分离。 |
| `docs/reference/calibration-profiles.zh-CN.md` | 有意采用不同实现 | 中文校准指南和严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级与明确降级证据；校准与单个 Work Item 质量路由分离。 |
| `docs/reference/calibration-session-model.ja.md` | 有意采用不同实现 | 目标明确保留校准 proposal、确认和仓库绑定事实，不静默引入通用交互 Session 或 checklist 权威；unknown 与人的责任保持可见。 |

本批次是语义责任对齐，不是源 wire 或命令字节对齐。目标使用一份共享外部 Runtime、仓库本地
`.ai/` 状态以及显式 `--repo`；provider 身份、托管 CI、签名、SBOM、provenance 和企业控制
仍属于委托证据。Cursor 采用方必须显式安装仓库本地 adapter，并重放持久化的
`work-item outcome` handoff；Runtime 无法强制 IDE 展开聊天面板。因此，Runtime 当前的输出和
生命周期入口门禁不等于自动向聊天发布。诊断 remediation、close-gap 便利命令和 controls
自动脚手架仍是独立的产品决策。

## WI-328：校准与能力事实文档切片

WI-328 逐个比对固定参考提交中的九个路径。其中五个按有意不同的实现登记；四个
能力矩阵/声明文档明确保留为 reference-only，因为 Rust 目标没有源项目的 public
claim checker 或 matrix。

| 参考源路径 | 分类 | Rust/adopter 对应与边界 |
| --- | --- | --- |
| docs/reference/calibration-session-model.md | implemented-different-by-design | repository-bound profile proposal、confirm 和 calibration facts 保留事实/证据边界，不引入通用持久化 Session。 |
| docs/reference/calibration-session-model.zh-CN.md | implemented-different-by-design | 中文 calibration/profile 路线保留 proposal、confirm、unknown 与人工责任边界。 |
| docs/reference/calibration-session.ja.md | implemented-different-by-design | 源十阶段 Session 只由目标显式 profile proposal/confirm 承接语义，不复制 Make/Python 编排。 |
| docs/reference/calibration-session.md | implemented-different-by-design | 源持久化 wizard 属于源专属编排；目标校准 read-only-first、repository-bound，策略变更必须人工确认。 |
| docs/reference/canonical-terminology.md | implemented-different-by-design | .ai/glossary.md、configuration 与 Outcome 参考页提供 canonical terms；治理 light 不等同于校准 lite，release 是 operation 而不是 profile。 |
| docs/reference/capability-claim-authoring.md | reference-only | 源 lexical claim checker 与 matrix front matter 绑定不是目标 Runtime gate；目标 registry 只报告观察事实和 exclusions，候选 WI-330 负责未来严格绑定。 |
| docs/reference/capability-evidence-freshness.md | reference-only | 目标已有 Work Item verification freshness，但没有独立 Capability Truth 行过期/portable-environment matrix；由候选 WI-330 定义边界。 |
| docs/reference/capability-truth-matrix.json | reference-only | 不复制源三十行 public matrix；capability_truth_registry 是观察能力 projection，不是 public claim authorization 或 adopter/provider proof。 |
| docs/reference/capability-truth-matrix.md | reference-only | 当前 capability/adoption 页面说明 observed fact、adopter、provider 与 enterprise 边界，不宣称源 matrix/checker。 |

四个 reference-only 结果是明确的产品边界，不是未登记遗漏。WI-330 已通过逐文件复核
闭合本批比较：源 claim checker、行级 freshness matrix 和 public matrix 都不是当前 Runtime
功能。未来 Rust 原生 claim/evidence gate 仍是可选产品决策，必须有单独的人工拥有 scope；不会
静默提升源 Python/V1 资产。

Cursor 采用方反馈是外部验证输入。当前 Runtime 已有稳定 lifecycle JSON、可重放
work-item outcome、close-before-next/readiness 检查及 fail-closed start/verification 绑定。
Runtime 无法强制 IDE 展开聊天面板，adapter/host 必须展示或重放持久化 handoff。诊断
remediation、controls 脚手架、close-gap 便利命令和 Makefile 集成仍是明确的后续/非目标。

## WI-330 能力真相边界决定

WI-330 再次逐一读取固定源版本的四个文件，并记录最终决定。目标的
`capability show` 仍然是 repository 与 snapshot 绑定的 projection；public claim 授权和
Capability Truth 行过期策略明确不属于当前 Runtime。

| 固定源路径 | 最终分类 | 决定与目标对应 |
| --- | --- | --- |
| docs/reference/capability-claim-authoring.md | reference-only | 不复制源 lexical trigger/claim-binding checker。文档 metadata 不授予证据；公开表述必须依赖当前的有界 evidence 并保留限制。对应：docs/capabilities.md、crates/cockpit-repository/src/lib.rs。 |
| docs/reference/capability-evidence-freshness.md | reference-only | Work Item receipt freshness 已存在，但源 Capability Truth 行过期和 portable-environment 策略不存在。对应：Runtime evidence validation、docs/reference/outcome-report.md。 |
| docs/reference/capability-truth-matrix.json | reference-only | 不把源三十行 matrix 当作 Rust wire format 或授权源。capability_truth_registry 只报告观察事实、adopter 状态和外部排除。对应：crates/cockpit-protocol/src/lib.rs、crates/cockpit-repository/src/lib.rs。 |
| docs/reference/capability-truth-matrix.md | reference-only | 目标 capability/adoption 页面说明 observed/evidence/provider/enterprise 边界，不宣传源 matrix 或 checker。对应：docs/capabilities.md。 |

这是产品边界决定，不是未登记遗漏。未来若人工拥有的 Work Item 引入 claim binding 或行级
freshness，必须先定义 Rust-native schema、evidence 生成、过期处理、三语 scope 和 adopter
验收，才能改变分类。

## WI-331 checks catalog 与 CI/release evidence

WI-331 逐一对比固定源版本中的以下两个路径。两者均按“有意不同实现”登记：Rust 目标保留源项目的质量与发布证据责任边界，但不复制源项目 Make、Python 或 V1 runtime。

| 固定源路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/checks-catalog.md` | implemented-different-by-design | `docs/reference/checks-catalog.*`、Contract-aware `gate` 路由、repository gate manifest、Rust workspace 检查、conformance/docs 检查以及 release/adopter 检查保留同一分层质量意图。本地检查与 provider/enterprise assurance 分离；dynamic light/standard/strict profile 会在 unknown 或 release-owned 控制项上升级。 |
| `docs/reference/ci-release-evidence.md` | implemented-different-by-design | `docs/reference/ci-release-evidence.*`、`.github/workflows/ci.yml`、`.github/workflows/release.yml`、发布分发检查和 adopter acceptance harness 绑定 provider job、commit/base/head、artifact、checksum、SBOM、provenance 与隔离 receipt。跳过或失败的 job 保持可见，PR 文本不会成为证据。 |

边界明确如下：目标 Runtime 负责 repository-local Contract 与 gate 决定；托管 CI、签名、SBOM/provenance provider 和企业审计系统负责其 delegated evidence。公开发布事实绑定不可变 tag 与下载 artifact。所有命令仍必须显式 `--repo`；源项目 Makefile、Python runner 或复制的 V1 runtime 不是目标要求。六份语言对应文档与 inventory 断言共同构成此批次的防遗漏记录。

## WI-332：P0 理解审查证据

WI-332 逐一读取固定参考源中的三个理解审查证据文件。三者均登记为
`reference-only`：它们是参考源仓库的历史桌面审查记录，审查者、日期、得分和语言结论
不能转移为本工程的证据。目标通过本地化首页、设计思想、架构、Agent workflow 和
文档 acceptance 检查保留六问读者路线，但不虚构独立的母语编辑审查，也不复制源证据
字节。这是语义读者路线对齐，不是宣称目标通过了源项目的审查。

| 固定源路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/comprehension-review-2026-08-14.md` | reference-only | `docs/README.md`、`docs/philosophy.md`、`docs/architecture.md`、`docs/reference/agent-workflow.md` 与 `tests/docs/documentation_acceptance.sh` 提供英文读者路线和结构检查；源审查结果不可移植。 |
| `docs/reference/comprehension-review-2026-08-14.zh-CN.md` | reference-only | `docs/README.zh-CN.md`、`docs/philosophy.zh-CN.md`、`docs/architecture.zh-CN.md`、`docs/reference/agent-workflow.zh-CN.md` 与文档 acceptance 检查提供中文路线；不宣称母语审查得分。 |
| `docs/reference/comprehension-review-2026-08-14.ja.md` | reference-only | `docs/README.ja.md`、`docs/philosophy.ja.md`、`docs/architecture.ja.md`、`docs/reference/agent-workflow.ja.md` 与文档 acceptance 检查提供日文路线；不宣称母语审查得分。 |

外部 Cursor 采用方反馈仍是独立的验证输入。Runtime 的稳定 lifecycle JSON、可重放的
human Outcome、readiness/start 门禁以及 verification 失效机制已在其他批次覆盖。本批不把
自动向 Cursor 聊天发布、`Makefile.ai`、close-gap 便利命令或 controls 模板静默提升为当前
parity。

### Cursor 采用方反馈评估（v0.2.33）

下面的采用方矩阵记录当前保证和明确边界，不是源工程 wire 兼容性声明。

| 反馈 | 当前边界 | 决定 |
| --- | --- | --- |
| 面向 Agent 的 Outcome 输出 | `finish`、`archive`、`close` 在 stdout 输出稳定 lifecycle JSON；`work-item outcome --json` 和带 repository context 的 MCP `work_item_outcome` 是可重放机器入口。 | Runtime 已实现。Cursor 必须在对话中展示 handoff；CLI 无法展开 IDE 面板。 |
| 下一个 Work Item 之前必须 close | readiness/lifecycle entry 拒绝 active Work Item、未关闭 archive、脏源文件、detached HEAD 以及未同步默认基线。 | 已实现且 fail-closed；`ready_on_base` 是显式状态。 |
| start 时机和 base 绑定 | start 拒绝开始前已有的非治理变更，并在实现前绑定显式 branch/worktree/base context。 | 已实现且 fail-closed。 |
| finalize/close 诊断 | 错误包含失败边界和恢复条件，但没有专用 `close-gap` 修复命令。 | 部分实现；更丰富的诊断属于未来有边界的产品决策。 |
| controls 脚手架 | 校验已声明的 controls/evidence，不发明 acceptance 决定，也不生成完整 controls 模板。 | 有意保持不生成治理决定。 |
| merge 后 close 恢复 | 显式 `finalize`、`finalize-verify`、`close` 以及 readiness/status 投影覆盖 lifecycle。 | 当前 lifecycle 是权威；`close-gap` 别名属于可选宿主 UX。 |
| Make 集成 | 目标使用显式 `--repo` CLI/MCP 和 provider adapter；源 `Makefile.ai` 编排不是协议要求。 | 不是 parity 遗漏；不复制源 Make/Python 编排。 |
| verification 失效 | lifecycle 边界校验 source snapshot、Contract、repository identity 和 evidence binding；源变更后必须重新 verification。 | 已实现且 fail-closed；归档 bytes 保持不可变历史事实。 |

任何未来 Runtime 变更都必须使用人拥有的有边界 Contract、测试、三语文档和发布版本验收；
采用方反馈不会变成未登记的承诺。

## WI-333：理解验证协议与参与者记录

WI-333 逐一读取固定参考源中的理解验证协议、严格响应 schema、六份匿名响应记录和结果文件。
12 个路径全部登记为 `reference-only`。这些文件描述的是参考源拥有的外部读者研究；参与者响应、
版本和样本结论不能移植为本工程证据。目标保留面向读者的文档路线，并将 Runtime evidence
校验与参与者研究分离。不复制任何响应字节或源结果，也不据此声称本工程已通过理解、发布、安全、
安保或企业研究。

| 固定源路径 | 分类 | 目标对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | reference-only | `docs/README.md`、`docs/reference/agent-workflow.md`、`docs/reference/outcome-report.md`；源资格、同意、访谈和审查协议仍属于外部研究。 |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | reference-only | `docs/README.zh-CN.md`、`docs/reference/agent-workflow.zh-CN.md`、`docs/reference/outcome-report.zh-CN.md`；不暗示目标已有参与者研究。 |
| `docs/reference/comprehension-validation-protocol.ja.md` | reference-only | `docs/README.ja.md`、`docs/reference/agent-workflow.ja.md`、`docs/reference/outcome-report.ja.md`；源伦理与资格不是 Runtime policy。 |
| `docs/reference/comprehension-validation-response.schema.json` | reference-only | `.ai/README.md`、`docs/reference/outcome-report.md`；参与者响应 schema 不是 Runtime Contract 或 verification-evidence schema。 |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | reference-only | `docs/README.md`、`docs/features/human-benefit-report.md`；历史响应、版本和 pseudonym 只绑定参考源。 |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | reference-only | `docs/README.md`、`docs/features/human-benefit-report.md`；不将参与者数据导入 `.ai/`。 |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | reference-only | `docs/README.ja.md`、`docs/features/human-benefit-report.ja.md`；源响应不是 adopter 或 Runtime evidence。 |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | reference-only | `docs/README.ja.md`、`docs/features/human-benefit-report.ja.md`；源版本绑定事实保持外部不可移植。 |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | reference-only | `docs/README.zh-CN.md`、`docs/features/human-benefit-report.zh-CN.md`；不为目标声称母语评分。 |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | reference-only | `docs/README.zh-CN.md`、`docs/features/human-benefit-report.zh-CN.md`；不复制原始参与者文本。 |
| `docs/reference/comprehension-validation-results.json` | reference-only | `docs/features/human-benefit-report.*`、`docs/reference/reference-file-comparison.*`；样本计数和有界结果仍绑定源版本。 |
| `docs/reference/comprehension-validation-results.md` | reference-only | `docs/features/human-benefit-report.md`、`docs/reference/outcome-report.md`；源限制不是目标 verification 或发布证据。 |

这个边界是有意设计的：adopter repository 可以继承目标的文档路线、Contract、evidence 和
Agent workflow，但不能继承另一仓库的人体参与者证据。未来若要开展研究，必须先建立独立的
同意、保留、隐私和 evidence Contract。

## WI-334：Evidence Binding 与 reuse 基础

WI-334 逐一读取固定参考源 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中的 10 个路径。
10 个路径全部判定为 `implemented-different-by-design`。Rust 目标把 content、diff、environment、
command、toolchain、policy、profile、Runtime、stage 和 runner identity 组合为严格的
`EvidenceContext`；不复制源 Python 模块，也不声称源 JSON/API 兼容。

| 固定源路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | implemented-different-by-design | `cockpit-evidence` 将 content identity 作为组合 context 的一部分；只有精确绑定才可进行 advisory reuse。 |
| `docs/reference/diff-bound-evidence-reuse.md` | implemented-different-by-design | `DiffIdentity`、repository snapshot facts 和 reuse 测试绑定 base/head 与 changed-path；不匹配必须 rerun。 |
| `docs/reference/environment-bound-reuse.md` | implemented-different-by-design | 显式绑定 Runtime/toolchain/environment/profile/policy/command/stage，不整体序列化进程环境。 |
| `docs/reference/evidence-binding-foundation.md` | implemented-different-by-design | 版本化 `ReusableReceipt` 校验 content-addressed identity、expiry、node 和 passed；不能绕过 protected 或 required checks。 |
| `scripts/ai_evidence_binding.py` | implemented-different-by-design | typed Rust structs、deny-unknown-fields 和确定性的 fail-closed 决定替代 Python builder/validator。 |
| `scripts/ai_diff_bound_reuse.py` | implemented-different-by-design | typed `DiffIdentity` 与 Git snapshot facts 替代源 helper，并保留 canonical path/revision mismatch 语义。 |
| `scripts/ai_environment_reuse.py` | implemented-different-by-design | 显式、有界的 environment 输入和 digest 字段替代源 adapter；不读取或持久化凭据。 |
| `tests/test_ai_evidence_binding.py` | implemented-different-by-design | Rust evidence/repository 测试覆盖 strict schema、篡改、mismatch、expiry、failed/protected node 与 rerun 决定。 |
| `tests/test_ai_diff_bound_reuse.py` | implemented-different-by-design | Rust evidence/Git 测试覆盖 clean/changed paths、canonical ordering、非法路径、policy mismatch、expiry 和不可变输入。 |
| `tests/test_ai_environment_reuse.py` | implemented-different-by-design | Rust evidence/executor 测试覆盖 environment/toolchain identity、stale/unknown receipt、protected execution 和 digest 校验。 |

本批建立的是语义责任对齐，不是源 wire 对齐。Reuse 只是优化/证据观察：只有精确 fresh binding
可以被考虑，治理、coverage、安全和 required-check gate 仍由调用方负责。Inventory、三语 ledger
和 WI-334 evidence 绑定这一决定；不引入源 participant、Python、Make 或 V1 artifact。

## WI-336：前五个治理文档路径

WI-336 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐个阅读下面五个路径。
结果区分可继承的治理责任、参考源专属报告、provider 自动化和历史清理工具。

| 固定源路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | reference-only | `docs/reference/reference-parity.md`、`docs/reference/outcome-report.md` 与每个 Work Item 的 archive 校验提供目标审计边界。源 WI-04..WI-13 聚合报告及不可观察的 conversation receipt 不是 Runtime 命令。 |
| `docs/reference/dependabot-intake.md` | not-applicable | Dependabot bot 分支接入是 provider 专属能力。通用 delegated provider evidence 和显式 Work Item source binding 在 `docs/reference/ci-release-evidence.md` 中描述，但不构成 Dependabot 授权路径。 |
| `docs/reference/deprecated-assets-registry.json` | reference-only | `.ai/README.md`、`docs/reference/agent-workflow.md` 和精确 resource finalization 保留显式审查清理与不可变历史边界；不提供源 registry 或 Make 扫描。 |
| `docs/reference/deprecated-assets.md` | reference-only | 过时命令链和 registry hygiene 的说明属于源文档。Rust 使用显式 `--repo`、Runtime lifecycle、不可变 archive 和 resource finalization，不声称存在 `check-deprecated-assets`。 |
| `docs/reference/derived-artifacts.md` | implemented-different-by-design | `docs/reference/outcome-report.md`、`docs/reference/verification-semantics.md`、`.ai/README.md` 和 typed Runtime projection 将 Contract/evidence/archive 事实与 status/Outcome 视图分开；不需要也不读取源 Python registry 作为 authority。 |

本批是语义责任比较，不是源命令或 wire 兼容性。Rust 不复制参考源 Python、Make target、
Dependabot workflow、删除 registry 或生成历史。每个 Work Item 的 archive 与面向人的 Outcome
仍是权威来源；derived view 不能授权后续决定。其余台账记录继续明确保持 deferred。

## WI-343：参考 inventory 基础对账

WI-339 已逐一比较下面五个固定参考路径，但机器 inventory 仍将它们标为
`deferred-next-batch`。WI-343 只把既有决定确定性登记到 inventory，不改变 Runtime 行为，也不复制源工具。

| 固定参考路径 | 分类 |
| --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` |
| `docs/reference/dependabot-intake.md` | `not-applicable` |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` |
| `docs/reference/deprecated-assets.md` | `reference-only` |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` |

三语 ledger 与生成 inventory 现在一致：240 条 implemented-different-by-design、4 条
not-applicable、30 条 reference-only、582 条 deferred，`migrate-gap` 为 0。这是 ledger
对账，不是源命令或 JSON-wire 兼容性声明。

## WI-342：文档、分发与企业边界批次

WI-342 在固定参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一读取以下 10 个路径。
其中 8 个判定为 `implemented-different-by-design`，2 个判定为
`reference-only`。目标保留读者路线、分发、权威边界和企业边界责任，但不复制
源 Python/Make 编排、源 adopter 记录或 provider 声明。

| 固定参考路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/distribution.md` | implemented-different-by-design | `docs/release/distribution.*` 与公开/N-1 adopter 验收脚本提供不可变 Release 验证、共享 Runtime 安装、repository 绑定、checksum/SBOM/provenance 与清理边界。 |
| `docs/reference/distribution.ja.md` | implemented-different-by-design | 日文路线由 `docs/release/distribution.ja.md` 与同一目标验收脚本承接；不复制源 Make/Python 安装细节或源字节。 |
| `docs/reference/documentation-architecture.md` | implemented-different-by-design | `docs/current/README.md`、getting-started/reference 路线、三语文档检查和本 ledger 保留 canonical layers、读者路线、owner 与拆分规则。 |
| `docs/reference/documentation-architecture.ja.md` | implemented-different-by-design | 日文 current/getting-started/reference 路线保留源读者地图和语言边界；`.ai/README.md` 与显式 Runtime 页面仍是 instruction 边界。 |
| `docs/reference/documentation-authority-boundary.md` | implemented-different-by-design | `.ai/README.md`、`AGENTS.md`、current/reference 路线、frontmatter 与文档 acceptance 将当前指示、可选参考和历史记录分开。 |
| `docs/reference/documentation-authority-registry.json` | implemented-different-by-design | 目标的显式路线与元数据检查替代源 topic registry；不引入全局 Agent 配置，也不把未验证的源 topic 声明为能力。 |
| `docs/reference/documentation-context-registry.json` | reference-only | 源计划/context 标签是源内部记录，不是可移植的 Runtime 权威或 adopter 证据。目标保留当前 `.ai` 指示和不可变 Work Item/archive 历史，不复制源 registry。 |
| `docs/reference/enterprise-control-checklist.md` | implemented-different-by-design | 三语 enterprise-governance、deployment-boundary 和 adopter-configuration 页面区分 repository facts、delegated evidence、retention/audit 责任与非认证声明。 |
| `docs/reference/enterprise-control-matrix.json` | reference-only | 源 observed-control 行不是可移植的合规结果。目标通过 delegated evidence 与 policy 路线要求当前外部 receipt，不复制源 `not_verified` 状态。 |
| `docs/reference/external-identity-boundary.md` | implemented-different-by-design | typed Rust authority/approval evidence、policy precedence、外部 evidence import、Contract 字段文档和企业页面保留身份等级，但不在本地认证个人。 |

两个 `reference-only` 记录不会被提升为目标能力：源 context 元数据和源 adopter 控制观察
不能转移为证据。这是语义/文档 parity，不是 JSON-wire parity。目标的对象工程边界仍然明确：
一个共享 Runtime、按 repository 隔离的 `.ai/` 状态、外部 provider evidence，以及不声称
组织级身份或合规能力。

本批完成后的台账为 5,119 条：4,262 条 `generated-history`、240 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条
`not-applicable`、30 条 `reference-only` 与 582 条 `deferred-next-batch`；
`migrate-gap` 仍为 0。582 条 deferred 仍是计划中的逐文件比较工作，不是 parity 声明。

## WI-344：参考文档第 14 批

WI-344 逐一读取以下五个固定参考文档。其中三个责任由 Rust 原生读者/Runtime
边界承接，两个属于源项目专属历史报告，不应成为目标能力或证据。

| 固定参考路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | implemented-different-by-design | `docs/reference/troubleshooting.md`、`docs/features/task-outcome-report.md`、`docs/reference/outcome-report.md` 和 typed recovery/Outcome service 提供仓库绑定的 failed-gate、recovery-condition、intervention、stop、resolution 与 next-action 报告。源九场景 Python report wire shape 仍单独排期。 |
| `docs/reference/final-north-star-acceptance.json` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、本 parity ledger 与 final-replacement harness 保留 20 个 dimension 及明确的外部 adopter/provider 限制，不导入源 decision bytes。 |
| `docs/reference/final-north-star-acceptance.md` | implemented-different-by-design | Design Philosophy、Product Boundary、Outcome 和 final-replacement acceptance 保留 North Star，并把本地检查与外部 evidence 分开。 |
| `docs/reference/final-wiii-remediation-closure-audit.md` | reference-only | 源 WIII 的 PR 身份、reviewer 和历史关闭声明不是可移植的目标证据；Rust 保留自己的 Work Item intelligence 与并行路线。 |
| `docs/reference/full-remediation-acceptance.md` | reference-only | 源 WI-01–WI-19 修复顺序是内部历史。目标保留自己的 evidence-bound acceptance 路线，不发布源进度或 Release 声明。 |

这是语义/文档 parity，不是源命令或 JSON-wire parity。配套源 recovery/acceptance
脚本和测试仍在各自文件比较中排期；对象工程边界仍是一个共享 Runtime、隔离的
repository 状态和独立绑定的 evidence。

当前台账为 5,119 条：4,262 条 `generated-history`、252 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条
`not-applicable`、34 条 `reference-only`、566 条 `deferred-next-batch`；
`migrate-gap` 为 0。deferred 数量是计划中的工作，不是 parity 声明。

## WI-345：治理成本与性能文档第 15 批

WI-345 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较以下五个文档。两个复杂度文档保持 `reference-only`，因为其中的 Python/Make scanner 和源阈值不是 Rust Runtime 行为。成本、性能预算和 profile/cost 分离由 Rust 原生、repository-bound 的投影承接，但边界更窄且明确为 advisory。

| 固定参考路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | reference-only | `docs/reference/governance-complexity.ja.md`、`docs/reference/governance-integrity-gate.ja.md` 和不可变 archive 规则记录边界；不复制源复杂度 scanner、Make target 或阈值。 |
| `docs/reference/governance-complexity.md` | reference-only | `docs/reference/governance-complexity.md`、`docs/reference/governance-integrity-gate.md` 与 `inspect/status/doctor` 保留 repository facts 和 archive integrity，不宣称源指标等价。 |
| `docs/reference/governance-cost-metrics.md` | implemented-different-by-design | `ai-cockpit diagnose --repo <repo> [--work-item <id>]`、typed `VerificationCostEstimate`/`VerificationCostObservation` 和 `docs/reference/verification-cost.md` 提供 identity-bound advisory facts；源 JSONL 阶段/等待解析和 wire shape 不是 Runtime 要求。 |
| `docs/reference/governance-performance-budget.md` | implemented-different-by-design | typed `PerformanceBaseline`/`PerformanceAssessment`、`tests/performance/regression_gate.sh` 与 `tests/performance/README.md` 执行明确的本地预算，不推导 P95，也不削弱必需验证。 |
| `docs/reference/governance-profile-cost-separation.md` | implemented-different-by-design | `docs/reference/governance-profile-cost-separation.md`、`ci-quality-gates.md` 与 `verification-route.md` 保持 light/standard/strict、operation/stage escalation、VerificationTier、EvidenceAssurance 和 cost 分离。 |

这是语义/文档 parity，不是源命令或 JSON-wire 兼容性。对象工程边界保持一致：一个共享 Runtime、显式 `--repo`、repository-local evidence、由 policy 拥有的路线要求，以及不能授权更弱治理结论的 advisory 成本/性能事实。

WI-345 后台账为 5,119 条：4,262 条 `generated-history`、246 条 `implemented-different-by-design`、1 条 `implemented-equivalent`、4 条 `not-applicable`、34 条 `reference-only`、572 条 `deferred-next-batch`；`migrate-gap` 为 0。572 条 deferred 仍是计划中的工作，不是 parity 声明。WI-346 的当前结果记录如下。

## WI-346：治理配置与 Cockpit 状态阅读

WI-346 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较以下六个参考文档。
六个文件均为 `implemented-different-by-design`：目标现在提供明确的三语阅读路线，但 Rust Runtime、
repository context 和 CI 边界与源 Make/Python 编排不同。

| 固定参考路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | implemented-different-by-design | `governance-profiles.ja.md`、`governance-profile-cost-separation.ja.md`、`ci-quality-gates.ja.md` 与 `verification-route.ja.md` 以日语保留比例化 profile、release 升级、成本/assurance 分离和 fail-closed 边界；不复制源调度字节。 |
| `docs/reference/governance-profiles.md` | implemented-different-by-design | 英文 profile 页面、成本分离、CI Gate 与 verification route 将 Light/Standard/Strict、release escalation、强制底线和显式 `gate --repo` 边界映射到目标。 |
| `docs/reference/governance-profiles.zh-CN.md` | implemented-different-by-design | 中文页面保留 profile、tier/assurance、成本和 override 边界，不把源 `make` 或 Python 命令说成 Rust 要求。 |
| `docs/reference/how-to-read-cockpit-status.ja.md` | implemented-different-by-design | 日语状态阅读页、`outcome-report.ja.md` 与 `commands.ja.md` 提供面向人的 handoff；Contract 原文和证据仍是权威。 |
| `docs/reference/how-to-read-cockpit-status.md` | implemented-different-by-design | 英文状态阅读页、`outcome-report.md` 与 `commands.md` 把源 reader 标签映射到 Rust Outcome 章节、颜色、停止条件和明确下一步。 |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | implemented-different-by-design | 中文页面提供同样的安全阅读顺序和证据边界；自动翻译不能改变 Contract 事实或产生批准。 |

六个页面明确区分 `VerificationTier`、`EvidenceAssurance` 和 advisory 成本观测；说明 🟢 仅表示可评审证据，
🟡 表示不完整或等待决定，🔴 表示必须停止，三者都不是 merge 或 release 授权。`unknown` 保持可见，不能靠猜测消除。
目标页面要求显式 `--repo`，保留 Contract 原文，并说明 MCP/宿主显示边界，使对象工程可以继承同一行为。

这是语义/文档 parity，不是源命令或 JSON-wire parity。WI-346 后当前台账为 5,119 条：4,262 条
`generated-history`、252 条 `implemented-different-by-design`、1 条 `implemented-equivalent`、4 条
`not-applicable`、34 条 `reference-only`、566 条 `deferred-next-batch`；`migrate-gap` 仍为 0。
566 条 deferred 仍是计划中的比较，不是 parity 声明。

## WI-347：Knowledge、输入信任、已安装生命周期与能力评估

WI-347 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较接下来的十个参考路径。
十个文件均为 `implemented-different-by-design`：目标增加了 Rust 原生的面向读者映射和明确限制，
而参考源的 Python/Make 编排、生成评估字节和 provider-global 行为仍在 Runtime 边界之外。

| 固定参考路径 | 分类 | Rust 对应物 / 有界决定 |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | implemented-different-by-design | `docs/features/human-benefit-report.md`、`docs/features/task-outcome-report.md` 与 `docs/reference/outcome-report.md` 保留决策视图顺序和禁止过度声明边界。 |
| `docs/reference/implementation-knowledge.ja.md` | implemented-different-by-design | 日语实现知识页和类型化 Knowledge 记录提供只读投影；不复制源过滤器和生成记录。 |
| `docs/reference/implementation-knowledge.md` | implemented-different-by-design | Rust Knowledge CLI/MCP 提供确定性仓库过滤器与 `KnowledgeV2Record`；更宽的日期/提交/supersession 查询明确不在本版本能力内。 |
| `docs/reference/implementation-knowledge.zh-CN.md` | implemented-different-by-design | 中文 Knowledge 页面说明当前过滤器、证据绑定和与源查询面的有界差异。 |
| `docs/reference/input-trust-dataflow.ja.md` | implemented-different-by-design | 日语来源说明映射到类型化 `FactOrigin`、可追溯派生和 fail-closed 观察。 |
| `docs/reference/input-trust-dataflow.md` | implemented-different-by-design | Rust 类型化事实、仓库快照观察和输入信任测试保留来源分类与注入边界，不宣称源 JSON wire 兼容。 |
| `docs/reference/input-trust-dataflow.zh-CN.md` | implemented-different-by-design | 中文页面说明来源、跨步骤和显式仓库边界。 |
| `docs/reference/installed-lifecycle.md` | implemented-different-by-design | 文档说明共享 Runtime 安装、显式 attach、不可变 Release 验收和独立迁移/回滚边界；源 installer Python/Make 仅作参考资料。 |
| `docs/reference/instruction-traceability.md` | implemented-different-by-design | inventory、comparison/parity 页面、Work Item 证据和关闭回执提供结构化正反向追溯；不复制源 remediation checker。 |
| `docs/reference/japanese-capability-assessment.json` | implemented-different-by-design | 三语能力页与可执行展示/对抗测试提供有界覆盖；源评估/语料字节和一般流畅度声明仍绑定参考源。 |

这是语义/文档 parity，不是源命令或 JSON-wire parity。对象工程边界保持一致：一个已安装 Runtime、显式 `--repo`、隔离的仓库事实/证据，以及外部 provider/enterprise assurance。Knowledge、provenance、安装、追溯和语言投影不能生成 authority、收益、批准或发布证据。

WI-347 后台账为 5,119 条：4,262 条 `generated-history`、262 条 `implemented-different-by-design`、1 条 `implemented-equivalent`、4 条 `not-applicable`、34 条 `reference-only`、556 条 `deferred-next-batch`；`migrate-gap` 仍为 0。556 条 deferred 仍是计划中的比较，不是 parity 声明。

## WI-348：验证、操作时策略与 provider 边界批次

WI-348 逐一比较固定提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 的后续十个路径。
其中七项责任在 Rust 中以不同方式实现，三项历史 provider/pre-release 记录保持
`reference-only`。Rust Core 增加严格的操作时评估器；它只是策略输入，不是执行器，
也不是 provider 权限。

| 固定参考路径 | 分类 | Rust 对应/边界决定 |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | 三语日语评估边界、Outcome、荒诞测试、安装和文档检查；不宣称一般流畅度。 |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | 验证/证据服务提供按比例路线、内容绑定复用、确定性的 partial 依赖、单调升级和可见 advisory 边界。 |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | 三语 Runtime 标签、marker、安全、unknown、决定、限制和下一步投影；Contract 值保留编写语言。 |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | 历史 provider 清单；当前状态必须重新获取，不能授权发布或 merge。 |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | 历史对账叙述；不复制到当前 status 或 `.ai/`。 |
| `docs/reference/operation-time-policy-reevaluation.ja.md` | implemented-different-by-design | Rust `OperationTimeRequest`/decision 评估器和严格回归测试；不复制源 Python trust 或 provider 执行。 |
| `docs/reference/operation-time-policy-reevaluation.md` | implemented-different-by-design | 同一操作时边界，显式绑定操作、目标、范围、权限、新鲜度、信任和影响。 |
| `docs/reference/operation-time-policy-reevaluation.zh-CN.md` | implemented-different-by-design | 同一 fail-closed 操作时评估器的中文读者页面。 |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | request-scoped `diagnose` 和 cost observation 报告执行/复用事实，不臆造 provider 等待、P95 或 assurance。 |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | 历史生成对齐收据；目标文档使用自己的仓库本地检查，不从源产物提升。 |

这是语义对齐，不是源 Python、Make、JSON wire 或 provider 状态对齐。更新后的台账
共 5,119 条：4,262 条 `generated-history`、269 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条
`not-applicable`、37 条 `reference-only`、546 条 `deferred-next-batch`；
`migrate-gap` 仍为 0。每个目标工程继续使用同一共享 Runtime、显式 `--repo`、仓库本地证据和对象/adopter 隔离。

## WI-368：发布前、荒诞测试、adopter 与 reference-impact 批次

WI-368 在固定提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较了 11 个路径。
其中 6 条是 `implemented-different-by-design`，5 条是 `reference-only`：

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | reference-only | 历史生成对齐记录；当前文档使用仓库本地 gate。 |
| `docs/reference/pre-release-documentation-review.json` | reference-only | 历史五策略审查；源发现不能授权目标发布。 |
| `docs/reference/project-test-timing-baseline.json` | implemented-different-by-design | 身份绑定的性能样本与 advisory budget；耗时不会降低验证。 |
| `docs/reference/provider-backed-governance-validation.md` | implemented-different-by-design | provider/hosted 控制保持为委托证据；本地检查不能证明它们。 |
| `docs/reference/real-absurd-injection-cases.{md,zh-CN.md,ja.md}` | implemented-different-by-design | canonical manifest 与 Rust 测试保留 15 个结构化 cases、12 个命名 RAI cases。 |
| `docs/reference/real-adopter-reference-validation.md` | implemented-different-by-design | 不可变公开 Release 的 adopter/upgrade 验收，含隔离生命周期和清理证据。 |
| `docs/reference/reference-impact-gate.{md,zh-CN.md,ja.md}` | reference-only | 源静态 scanner/schema/Make surface 未提供；操作时策略是更窄的已声明事实边界。 |

本批还修正了 Standard profile 的措辞，不再暗示存在静态 reference-impact scanner。
参考源荒诞测试三语页面对命名场景数量不一致；目标以 manifest 作为机器事实并保留差异。
这是语义 parity 与明确边界文档，不是源命令或 JSON-wire 兼容。

## WI-378：参考文档第 17 批

WI-378 在固定参考提交上逐一比较了下一批十个 deferred 路径。其中九项责任由 Rust-native 三语文档和现有 Runtime/测试承担；一个生成的计划追溯文件保持 `reference-only`。本批不复制源 Python、Make、Provider 配置或历史修复决定。

| 固定参考路径 | 分类 | Rust 对应/边界决定 |
| --- | --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | reference-only | `docs/reference/instruction-traceability.md` 和机器 inventory 说明当前追溯边界；源生成的历史计划指令不是目标 authority。 |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | 三语 `docs/reference/repository-workflow.*`、`.ai/README.md`、`AGENTS.md` 保留显式仓库上下文、串行 Work Item、评审 PR、close 和清理语义。 |
| `docs/reference/schemas.md` | implemented-different-by-design | 三语 `schemas.*`、typed Protocol/repository validators 和不可变 evidence/decision 边界映射记录族，不宣称源 wire 兼容。 |
| `docs/reference/test-architecture.md` | implemented-different-by-design | 三语 `test-architecture.*`、CI quality route、conformance manifest、release/adopter harness 和负向优先测试说明分层证据与外部限制。 |
| `docs/reference/test-weakening-guard.ja.md` | implemented-different-by-design | 日语 Rust-native weakening 路由、基于 snapshot 的治理 Signal 与回归；不发布源 Python/Make surface。 |
| `docs/reference/test-weakening-guard.md` | implemented-different-by-design | 英语 Rust-native weakening 路由、保守路径处理、动态 profile 边界和恢复条件。 |
| `docs/reference/test-weakening-guard.zh-CN.md` | implemented-different-by-design | 中文 Rust-native weakening 路由、fail-closed unknown、按比例分析和明确非声明。 |
| `docs/reference/troubleshooting.ja.md` | implemented-different-by-design | 日语 stop-state/recovery、命令参考、installed-lifecycle 边界和文档检查替代源 wizard/Make 指令。 |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | 英语 stop-state/recovery 路由，明确 toolchain、adopter、active Work Item 和证据保留边界。 |
| `docs/reference/upgrade.ja.md` | implemented-different-by-design | 日语 Runtime upgrade 与 repository migration 的区分、不可变 Release、回滚和历史保留规则。 |

更新后的台账仍为 5,119 条：4,262 条 `generated-history`、284 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条
`not-applicable`、43 条 `reference-only`、525 条 `deferred-next-batch`；
`migrate-gap` 为 0。Deferred 集合仍是计划中的比较，不是 parity 声明。

## WI-379：参考文档第 18 批

WI-379 在固定参考提交上逐一比较下一批十个 deferred 路径。八项责任由 Rust-native
三语文档承担，两个历史审计文件保持 `reference-only`。本批不添加 Runtime 代码，也不
复制源 Python、Make、Provider 配置或生成历史。

| 固定参考路径 | 分类 | Rust 对应/边界决定 |
| --- | --- | --- |
| `docs/reference/upgrade.md` | implemented-different-by-design | 三语 `upgrade.*`、`installed-lifecycle.*` 以及 migration/conflict/rollback 边界；源 installer 命令仅作说明。 |
| `docs/reference/verification-evidence-reuse-runtime.md` | implemented-different-by-design | `verification-evidence-reuse-runtime.*`、`verification-route.*`、`verification-semantics.*`、typed identity-bound receipt、受保护节点执行和可观测复用指标。 |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | `verification-evidence-reuse.*`、`verification-cost.*`、`verification-planner.*`；精确绑定/失效和 advisory 调用次数边界。 |
| `docs/reference/verification-fixture-boundary.md` | implemented-different-by-design | `verification-fixture-boundary.*` 与 repository-native 测试；本地 fixture 排除 Runtime/cache 状态，不能证明 provider/adopter evidence。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | reference-only | 历史生成的 V1 审计 bytes；当前目标事实来自 pinned inventory、Work Item archive、evidence 和三语追溯页面。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | reference-only | 绑定源 Python/Make evidence 的历史叙述，不复制也不作为当前目标 authority。 |
| `docs/reference/wiii-v2-integration-audit.md` | implemented-different-by-design | `wiii-v2-integration-audit.*`、Rust `status`/intelligence 投影、显式 schema/source identity 检查及无 scheduler/provider 声明。 |
| `docs/reference/work-item-intelligence-performance-baseline.md` | implemented-different-by-design | `work-item-intelligence-performance-baseline.*`、`diagnose` 和 advisory 成本/性能观测；不声称源 benchmark 数字。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` | implemented-different-by-design | `work-item-lifecycle-closure.*`、`repository-workflow.*` 与 Runtime `finalize`/`close` receipt，精确绑定 PR/base/branch/worktree 清理。 |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | 英语 Rust-native close 路径与 recovery 边界；源 `make`/Python 编排不是命令要求。 |

这是语义/文档 parity，不是源命令、JSON-wire 或 provider 状态兼容。对象/adopter 边界
保持为一个共享 Runtime、显式 `--repo` 和隔离的 repository facts、Work Item、evidence、
knowledge、snapshot。WI-379 后台账为 4,262 条 `generated-history`、292 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条 `not-applicable`、
45 条 `reference-only`、515 条 `deferred-next-batch`；`migrate-gap` 仍为 0。

## WI-386：参考文档第 19 批

WI-386 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较了四个 deferred 文档。
其中两个历史/内部文档保持 `reference-only`；Roadmap 与 Security Boundary 的责任由当前 Rust-native
文档承载。本批不复制源 Python、Make 命令、Provider 配置、历史 GO/NO-GO 结论，也不把未来路线
里程碑当作已发布能力。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/review-final-evidence.md` | reference-only | 绑定源专用 `make` 检查和历史审查状态的生成 R11 证据索引。当前由 `final-replacement-acceptance.md`、`ci-release-evidence.md` 与仓库本地 Work Item evidence 生成新的身份绑定事实，不复制历史 GO/NO-GO。 |
| `docs/review-remediation-backlog.md` | reference-only | 内部 R0–R11 整改清单及 Python/Make 执行计划。当前边界由 `repository-workflow.md`、`governance-integrity-gate.md` 和本比较台账维护；源计划不是当前 authority。 |
| `docs/roadmap.md` | implemented-different-by-design | `docs/philosophy.md`、`docs/architecture.md`、`docs/capabilities.md` 保留使命、证据治理、Intent、人类控制、Repository Intelligence 与组织策略方向。历史 V1–V4 里程碑和源文案不作为已发布能力声明。 |
| `docs/security-boundaries.md` | implemented-different-by-design | `docs/security/threat-model.md`、`docs/reference/input-trust-dataflow.md`、`docs/reference/operation-time-policy-reevaluation.md`、`docs/security/adversarial-validation.md` 保留内容/权限分离、确定性 fail-closed、高风险重评估和限制。源 classifier 实现不复制。 |

这是语义/文档 parity，不是源命令、JSON-wire 或 Provider 状态兼容。所有对象/adopter 工程从共享
Runtime 继承 Rust-native 文档边界，但 repository facts、Work Item、evidence、knowledge 和 snapshot
仍在显式 `--repo` 下隔离。WI-386 后台账为 4,262 条 `generated-history`、294 条
`implemented-different-by-design`、1 条 `implemented-equivalent`、4 条 `not-applicable`、47 条
`reference-only`、511 条 `deferred-next-batch`；`migrate-gap` 仍为 0。

## WI-387：参考文档第 20 批

WI-387 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较接下来的四个安全与供应链文档。
其责任由 Rust-native 的安全、信任流、发布证据和分发文档承载。本批保留有界的仓库治理响应与外部控制责任边界，
不声称提供通用提示词注入检测器，也不由 Runtime 生成签名、SBOM、provenance 或 provider assurance。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | implemented-different-by-design | `docs/security/adversarial-validation.ja.md`、`docs/reference/input-trust-dataflow.ja.md`、`docs/reference/operation-time-policy-reevaluation.ja.md` 保留日语注入边界、操作时 fail-closed 复核和外部控制限制。 |
| `docs/security/injection-boundary.md` | implemented-different-by-design | `docs/security/adversarial-validation.md`、`docs/reference/input-trust-dataflow.md`、`docs/reference/operation-time-policy-reevaluation.md` 保留有界仓库治理响应；不可信文本仍是数据，源页面不复制为通用检测器声明。 |
| `docs/security/injection-boundary.zh-CN.md` | implemented-different-by-design | `docs/security/adversarial-validation.zh-CN.md`、`docs/reference/input-trust-dataflow.zh-CN.md`、`docs/reference/operation-time-policy-reevaluation.zh-CN.md` 保留中文边界、确定性 fail-closed 处理和非声明。 |
| `docs/security/supply-chain.md` | implemented-different-by-design | `docs/security/threat-model.md`、`docs/reference/ci-release-evidence.md`、`docs/release/distribution.md`、`docs/getting-started/security-release-verification.md` 保留委托式供应链证据责任与精确制品绑定；外部信任根仍在 Runtime 之外。 |

WI-387 后台账为 4,262 条 `generated-history`、298 条 `implemented-different-by-design`、1 条
`implemented-equivalent`、4 条 `not-applicable`、47 条 `reference-only`、507 条
`deferred-next-batch`；`migrate-gap` 仍为 0。每个已 attach 的对象/adopter 工程都继承相同的
Rust-native 安全与供应链边界，而 repository facts 与 evidence 继续由显式 `--repo` 隔离。

## WI-388：参考文档第 21 批

WI-388 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较六个 deferred 文档。
其责任已由 Rust-native 威胁模型、采用、发布证据、安装和排查路径承载。本批记录分布式对应关系与证据边界，
不复制源命令或历史稳定性结论。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | `docs/security/threat-model.md`、`.zh-CN.md`、`.ja.md` 保留保护资产、信任边界、fail-closed 威胁和外部控制限制；不声称能识别所有恶意意图或认证企业安全。 |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、`docs/getting-started/standard-adoption-guide.md`、`docs/reference/ci-release-evidence.md` 与 `tests/release/adopter_acceptance.sh` 分布承载模板、采用、生命周期和证据类型边界；模板单独运行不能升级为外部稳定性证明。 |
| `docs/troubleshooting.md` | implemented-different-by-design | 三语 `docs/reference/troubleshooting.*` 提供停止状态、恢复、证据保留和显式 repository-bound 命令，而不是仅兼容性跳转页。 |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | `docs/getting-started/installation.ja.md`、`installation-security.ja.md` 和 `docs/reference/troubleshooting.ja.md` 保留不确定即停止、严格 Release 验证和显式 attach，不复制源 wizard 命令。 |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | `docs/getting-started/installation.md`、`installation-security.md` 和 `docs/reference/troubleshooting.md` 保留不确定即停止、严格 Release 验证和显式 attach，不静默选择移动分支或旧制品。 |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | `docs/getting-started/installation.zh-CN.md`、`installation-security.zh-CN.md` 和 `docs/reference/troubleshooting.zh-CN.md` 保留中文恢复路径、严格制品绑定和显式 repository context。 |

这是语义/文档 parity，不是源命令、JSON-wire 或 provider 状态兼容。每个已 attach 的对象/adopter 工程都从共享
Runtime 继承相同的威胁、采用、安装和恢复边界，而 repository facts 与 evidence 继续由显式 `--repo` 隔离。
WI-388 后台账为 4,262 条 `generated-history`、304 条 `implemented-different-by-design`、1 条
`implemented-equivalent`、4 条 `not-applicable`、47 条 `reference-only`、501 条 `deferred-next-batch`；
`migrate-gap` 仍为 0。

## WI-389：参考文档第 22 批

WI-389 在固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较六个延后参考文档。卸载指南由已安装 Runtime 生命周期路线承载，升级指南由 Rust 原生升级参考承载。本批保留先提案后写入、负责人确认、不可变 Release 绑定、回滚、冲突停止和显式 active 恢复边界，不复制源 installer 命令。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.ja.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/troubleshooting/uninstall.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/troubleshooting/uninstall.zh-CN.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.zh-CN.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/upgrade.ja.md` | 有意采用不同实现 | `docs/reference/upgrade.ja.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |
| `docs/upgrade.md` | 有意采用不同实现 | `docs/reference/upgrade.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |
| `docs/upgrade.zh-CN.md` | 有意采用不同实现 | `docs/reference/upgrade.zh-CN.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |

这是语义/文档对等，不是源命令、JSON wire 或 provider state 兼容。每个 attach 的对象/采用方工程都从共享 Runtime 继承同一卸载、升级、回滚和恢复边界；工程事实和证据仍由显式 `--repo` 隔离。WI-389 后清单为 4,262 个 `generated-history`、310 个 `implemented-different-by-design`、1 个 `implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 495 个 `deferred-next-batch`；`migrate-gap` 仍为零。

## WI-390：参考 Work Item 编写指南

WI-390 逐段比较固定源文件 `docs/work-item-style-guide.md`。其面向读者的指导由三语 Rust-native
编写指南承载，并链接到 Contract 与 repository workflow 参考。本批保留先说明结果、明确问题和边界、
可观察验收、由人拥有的治理决定、足够小的流程、可执行验证以及先文档后 schema 等原则。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/work-item-style-guide.md` | 有意采用不同实现 | `docs/reference/work-item-style-guide.md`、`.zh-CN.md` 和 `.ja.md`，由 reference index 链接，并以 `contract-fields` 与 `repository-workflow` 为上下文。页面保留由人拥有的 intent/problem/constraints/rationale、显式 scope/non-goals、机器可检查验收、可执行验证、相称 profile 和对象/采用方工程继承；不复制源元数据、Python/Make 命令、installer 行为或 Runtime 实现。 |

这是语义/文档对等，不是源命令或 JSON wire 兼容。共享 Runtime 仍在每个对象工程之外；每个 attach 的仓库通过自己的 `.ai/` 与 adapter 继承同样的面向读者边界，而 Contract、evidence、knowledge 和 repository identity 继续由显式 `--repo` 隔离。WI-390 后清单为 4,262 个 `generated-history`、311 个 `implemented-different-by-design`、1 个 `implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 494 个 `deferred-next-batch`；`migrate-gap` 仍为零。

## WI-391：C# 适配示例

WI-391 在固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐节比较
`examples/csharp/README.md`。源文件的四项关注点——安装、.NET 质量检查与 coverage 边界、Contract
设计以及 guideline 合规 evidence——由三语 Rust 原生 C# 适配页及现有安装、Contract、verification
参考共同承载。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `examples/csharp/README.md` | 有意采用不同实现 | `docs/reference/csharp-adaptation.md`、`.zh-CN.md`、`.ja.md`，并链接共享 Runtime 安装、Contract 字段和 verification route。保留源语义，但 `install.sh`、`Makefile.ai.stack`、源 guard/Python 编排以及旧 JSON-wire 示例按设计保持外部或不兼容。 |

这是语义/文档对等，不是 C# 工具链支持或第二技术栈 adopter 验收声明。未来 C# adopter 回执必须使用不可变公开 Release 与自己的 repository context。共享 Runtime 在 adopter 外安装一次，`.ai/`、Contract、evidence 和项目 policy 仍在 repository 内，由显式 `--repo` 隔离。
WI-391 后清单为 4,262 个 `generated-history`、312 个 `implemented-different-by-design`、1 个
`implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 493 个
`deferred-next-batch`；`migrate-gap` 仍为零。

## WI-392：Android fixture 适配

WI-392 在固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较四个 Android fixture 文件。
Kotlin 源码和测试语义映射到 adopter 自己的路径与命令；fixture 元数据和 Gradle 拓扑映射为 Project
Profile/Observer 事实，同时明确 Unknown 边界。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt` | 有意采用不同实现 | `docs/reference/android-fixture-adaptation.zh-CN.md` 将源路径映射到显式 Contract scope，Kotlin 执行仍由 adopter/provider 负责。 |
| `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt` | 有意采用不同实现 | 适配指南将 `kotlin.test` 断言映射为 owner 确认的 Gradle verification 命令；测试文件不能证明 SDK/device/CI 已准备。 |
| `examples/fixtures/android-app/fixture.json` | 有意采用不同实现 | Project Profile/Observer 可记录 stack/toolchain/platform/path 事实；`installerStack` 不是 Runtime 安装契约，platform 标签也不是证据。 |
| `examples/fixtures/android-app/settings.gradle.kts` | 有意采用不同实现 | Gradle repository/module 拓扑作为有界上下文记录；依赖、SDK、credential、network 和 hosted-CI readiness 在有证据前保持 Unknown。 |

这是语义/文档对等，不是 Android 工具链支持、构建执行或源 JSON wire 兼容。安装有意采用每个 adopter
之外的一份不可变共享 Runtime，并显式 `attach --repo`；参考 fixture 的 Gradle 文件、SDK 安装和 installer
行为不复制。WI-392 后清单为 4,262 个 `generated-history`、316 个 `implemented-different-by-design`、1 个
`implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 489 个 `deferred-next-batch`；
`migrate-gap` 仍为零。

## WI-393：Flutter fixture 适配

WI-393 在固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较四个 Flutter fixture 文件。
Dart 源码和测试语义映射到 adopter 自己的路径与命令；fixture 和包元数据映射为 Project Profile/Observer
事实，同时明确 Unknown 边界。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `examples/fixtures/flutter-app/fixture.json` | 有意采用不同实现 | `docs/reference/flutter-fixture-adaptation.zh-CN.md` 将项目类型、stack、toolchain、platform 和安全/测试路径映射为有界 Profile/Contract 事实。`installerStack` 不是 Runtime 安装契约。 |
| `examples/fixtures/flutter-app/lib/main.dart` | 有意采用不同实现 | `greeting()` 源路径属于 adopter 的 Contract scope；Dart 执行由所有者/Provider 负责，Runtime 不推断。 |
| `examples/fixtures/flutter-app/pubspec.yaml` | 有意采用不同实现 | 包名称和 Dart SDK 范围是可观察元数据；SDK、依赖、网络和 lockfile 就绪状态在证据前保持 Unknown。 |
| `examples/fixtures/flutter-app/test/widget_test.dart` | 有意采用不同实现 | `flutter_test` 断言映射为所有者确认的 Provider 命令；文件本身不能证明 SDK、平台 runner、插件或托管 CI 就绪。 |

这是语义/文档对等，不是 Flutter 工具链支持、构建执行或源 JSON wire 兼容。安装有意采用每个 adopter
之外的一份不可变共享 Runtime，并显式 `attach --repo`；Flutter SDK/包安装和参考源安装实现不复制。WI-393
后清单为 4,262 个 `generated-history`、320 个 `implemented-different-by-design`、1 个
`implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 485 个
`deferred-next-batch`；`migrate-gap` 仍为零。

## WI-394：iOS Swift Package fixture 适配

WI-394 在固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较四个 iOS Swift Package
fixture 文件。Swift Package 拓扑、源码和 XCTest 语义映射到 adopter 自己的路径与命令；fixture 元数据
映射为 Project Profile/Observer 事实，同时明确 Unknown 边界。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `examples/fixtures/ios-swift-package/Package.swift` | 有意采用不同实现 | SwiftPM product/target 拓扑是对象工程/Provider 所有的构建元数据；Runtime 不推断 SDK/Xcode 就绪。 |
| `examples/fixtures/ios-swift-package/Sources/AppCore/AppCore.swift` | 有意采用不同实现 | `greeting()` 源路径属于对象工程 Contract scope；Swift 执行仍由 Provider 负责。 |
| `examples/fixtures/ios-swift-package/Tests/AppCoreTests/AppCoreTests.swift` | 有意采用不同实现 | XCTest 断言映射为所有者确认的 `swift test` 或 Xcode 命令；文件本身不能证明 SDK、模拟器、签名或托管 CI 就绪。 |
| `examples/fixtures/ios-swift-package/fixture.json` | 有意采用不同实现 | Project Profile/Observer 可记录 package/toolchain/platform/path 事实；`installerStack` 和 `macos` 是元数据，不是共享 Runtime 安装或执行证据。 |

这是语义/文档对等，不是 Apple 工具链支持、构建执行或源 JSON wire 兼容。安装有意采用每个 adopter
之外的一份不可变共享 Runtime，并显式 `attach --repo`；SwiftPM/Xcode 安装、SDK 选择和源安装器行为不复制。
WI-394 后清单为 4,262 个 `generated-history`、324 个 `implemented-different-by-design`、1 个
`implemented-equivalent`、4 个 `not-applicable`、47 个 `reference-only` 和 481 个
`deferred-next-batch`；`migrate-gap` 仍为零。

## WI-421：mixed-monorepo fixture 边界

WI-421 在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较
`examples/fixtures/mixed-monorepo/` 下的五个文件。它们是可执行的业务示例，
不是 Rust Runtime 代码或可移植的企业证据，因此五个路径均记录为
`reference-only`，并明确对象工程边界。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `fixture.json` | reference-only | 记录示例的 Python/Node 标记、平台和 safe/test 路径。Project Observer/Profile 可以记录实际观察到的事实，但 Runtime 不从此文件推断工具链能力或安全范围。 |
| `package.json` | reference-only | 包元数据只是示例业务输入。Node 安装、依赖、脚本和执行仍由对象工程/provider 负责。 |
| `pyproject.toml` | reference-only | Python 打包元数据不是可移植 Contract 或 Runtime 依赖；Python 安装、依赖和测试命令需要对象工程明确提供证据。 |
| `services/api/app.py` | reference-only | 健康函数是业务示例代码，不是治理逻辑。Runtime 可以绑定对象工程声明的 argv 结果，但不会携带或推断 Python 行为。 |
| `services/api/tests/test_app.py` | reference-only | pytest 断言只是 fixture 证据。对象工程必须声明并运行自己的验证命令；源测试不会被提升为目标证据。 |

本批保留可迁移的治理含义——观察事实、显式范围、provider 执行责任和证据绑定——
但不复制 mixed fixture、Python/Node 工具链、安装行为或源 JSON wire。每个 attach 的
对象/采用方工程都从共享 Runtime 继承相同的 Contract、lifecycle、evidence、knowledge
和人类 Outcome 控制；仓库 identity 与事实仍在显式 `--repo` 下隔离。本批不声明
mixed-stack 工具链支持，也不构成第二技术栈 adopter 验收。WI-421 后台账为 4,262 条
`generated-history`、324 条 `implemented-different-by-design`、1 条 `implemented-equivalent`、
4 条 `not-applicable`、65 条 `reference-only` 与 463 条 `deferred-next-batch`；`migrate-gap` 仍为 0。

## WI-475：Outcome、事件与质量门参考源比对

WI-475 在维护中的本地参考源提交
`fde3380f81fea5fd2e288f7a8849f737dc074060` 上重新逐节阅读七个发生变化的路径。
本次记录有界的语义决定；不把源 Python/Make/provider bytes 复制到 Rust 仓库。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | Rust `OutcomeV2`/`humanHandoff`、Task Outcome 参考和 CLI/MCP 测试保留确定性人类投影、evidence 计数、归档归属和明确非声明。源 `ai-finish`/`check-ai-pr` 报告文件仍是 source/provider 表面。 |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | 中文读者路线通过本地化 Rust 参考保留相同的投影、计数、归档和非声明语义；不复制源报告命令或字节。 |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | 日文读者路线通过本地化 Rust 参考保留相同的确定性投影和 evidence 边界；源报告命令和字节不属于目标 Contract。 |
| `docs/maintainers/task-outcome-events.md` | implemented-different-by-design | 三语 Rust 事件参考、严格事件模型和回归覆盖追加式历史、修正/取代、fingerprint、关系、隐私和 provider evidence 边界。源 Python generator/validator/renderer 仅作为语义材料。 |
| `docs/operations/quality-gates.md` | implemented-different-by-design | Rust Contract 感知 CI gate、governance-integrity 检查、审核过的 gate manifest、CI/release 表面和 runner 测试保留动态 profile、shadow 对照、证据归属、超时、性能样本和可追溯性。源 `make quality`、`Makefile.ai.stack` 与 Python runner bytes 仍是 adopter/provider 边界。 |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | 中文 CI 参考和 gate manifest 通过显式 `--repo` 保留源质量层级、动态路由、分片/evidence、超时、性能和追踪语义；不向 adopter 安装源 Make/Python 配置。 |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | 日文 CI 参考和 gate manifest 通过显式 repository context 保留源质量层级、动态路由、分片/evidence、超时、性能和追踪语义；不复制源 Make/Python 配置。 |

本次没有发现实现遗漏。目标有意将这些责任放在 `docs/features`/`docs/reference` 和
typed Runtime/gate 表面，而不是新增源专属的 `docs/maintainers` 或 `docs/operations` 文件。
所以同路径文件缺失是明确的布局边界，不是未审查遗漏。Contract intent 与 acceptance criteria
保持 authored language；本地化只改变展示，不改变治理事实。

共享 Runtime 只在 adopter 外部安装一份。每个 attach 的对象/采用方工程通过显式 `--repo`
继承自己独立的 `.ai/`、Contract、evidence、knowledge 和 adapter context；不复制源 Python
模块、Make target、报告文件或质量配置。WI-475 台账保留七个路径的
`sourceChangedSincePrevious` 与此前分类，并移除其 deferred 状态。当前为 4,262 条
`generated-history`、303 条 `implemented-different-by-design`、1 条 `implemented-equivalent`、
4 条 `not-applicable`、66 条 `reference-only`、483 条 `deferred-next-batch`；`migrate-gap` 仍为 0。
