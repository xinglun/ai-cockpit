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

本页说明 Rust 工程如何与公开参考源逐个文件比较。参考源是规格和行为语料，
不是应复制到 Rust Runtime 的目录。

## 固定基线

- 参考源：[spirex-ds-dev/ai-cockpit-template](https://github.com/spirex-ds-dev/ai-cockpit-template)，提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
- Rust 比较基线：[xinglun/ai-cockpit](https://github.com/xinglun/ai-cockpit) 的 `origin/main`，提交 `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`。
- 比较时使用的 Runtime：`ai-cockpit 0.2.33`，binary SHA256 为 `eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。

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

在固定的 v0.2.33 比较基线上，台账共有 5,119 条记录：4,262 条
`generated-history`、176 条 `implemented-different-by-design`、1 条
`implemented-equivalent`、3 条 `not-applicable` 与 677 条
`deferred-next-batch`。deferred 记录仍是待比较工作，不是 parity 声明。
capability/profile slice 已没有 `migrate-gap`：

1. `.ai/project/adopter-capability-manifest.json` 由 Runtime registry 表达，installer-surface
   仍是外部边界。
2. `.ai/project/capabilities.json` 由严格 Rust-native declaration 与显式 operation mapping 表达。
3. `.ai/project/success_criteria.json` 作为不具授权能力、绑定 snapshot 的可见 projection 表达。
4. `.ai/project_profile.yaml` 由 `.ai/project.json` 与严格 JSON `profile-policy.json` projection 表达。

治理入口、getting-started 路线、CI/release 边界与 capability/profile projection 已按该基线审阅；
以上四条是有界的 Rust-native counterpart，677 条 deferred 语义比较仍是后续工作。

WI-274 只将目标 checkout metadata 和 canonical comparison snapshot 重新绑定到已审阅的
默认分支提交。WI-273 保持为不可变的失败交付记录：其首次提交无法证明 parity 登记先于
verification evidence，因此 successor 会隔离保留这段历史，不会改写它。

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

WI-302/WI-304 批次没有发现 `migrate-gap`。台账现在为：4,262 条
`generated-history`、176 条 `implemented-different-by-design`、1 条
`implemented-equivalent`、3 条 `not-applicable`、677 条 `deferred-next-batch`。
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
