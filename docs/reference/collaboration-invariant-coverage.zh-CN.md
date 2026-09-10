---
author: AI Cockpit maintainers
title: "协作不变量覆盖情况"
description: "将十条协作语言语义不变量映射到既有的自动化测试覆盖，引用确切的测试文件与函数，并准确说明剩余边界。"
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-781-trust-diagnostics
---

# 协作不变量覆盖情况

本页回答:对于 WI-679 协作语言契约(`docs/reference/collaboration-language-contract.md`,
已通过 PR #675 合并)中陈述的十条语义不变量,今天是否已经有自动化测试在
强制执行?如果有,是哪一个?其目的是防止重复建设测试基础设施(优先复用),
并如实说明哪些不变量尚无自动化交叉检查。

以下每一条引用都是在撰写本页时直接阅读当前测试源码得到的,没有任何一条
仅凭测试名称推断。

**本页取代此前的 WI-681 草稿。** WI-681 未合并即被关闭,原因是另一个并行
运行的代理独立使用了相同的短编号(`WI-681-wi674-doc-promotion`,已通过
PR #677 合并)——这是一次真实的多代理 WI 编号冲突,而非内容缺陷。本次
重新交付(WI-740)同时修正了该草稿中的一处判断:不变量8原本被标记为
"未发现自动化测试",但更仔细阅读后发现已有测试覆盖其核心主张(见下方第8
行)。WI-681 指出的缺口不变量5与9,已分别由 WI-682(PR #679)与 WI-710
(PR #702)独立关闭,因此本页也将其标记为"是"。

## 覆盖情况表

| # | 不变量 | 今天是否自动化? | 测试 | 断言内容 |
| --- | --- | --- | --- | --- |
| 1 | 验证通过 ≠ 全面受理/合并授权 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | 让同一个 Work Item 经历 `finish`(绿)→`archive`(黄,终结尚未完成)→记录一次 `Deleted` 终结 →`close`(需要显式 `--human-decision`),并在每一步断言交接文本与稳定 JSON 都不会把这些状态坍缩成单一的"已完成"。 |
| 2 | 空记录 ≠ 无风险 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | 渲染一个风险/测试削弱字段为空的 Outcome,断言人类可读文本从不会因数据缺失而声称"未发现风险"或等价的正面结论。 |
| 3 | 未知事实不能由表达层补全 | **是(部分)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | 遍历 `tests/adversarial/manifest.json` 中的语义案例(目前 15 条)并通过 `evaluate()` 断言评估结果绑定于案例数据而非措辞。仅覆盖该清单中已收录的对抗性措辞,并非穷尽。 |
| 4 | 历史记录不能无依据地变成当前的通过/失败 | **是** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`、`::archived_report_tamper_is_red_and_not_reprojected_as_verified` | 断言 `runtime_historical` 记录保留其历史措辞(不会出现本应用于当前失败的"Repair the missing evidence"文案),以及被篡改的归档报告会渲染为红色,而不是被静默地重新投影为已验证。 |
| 5 | 同一事实在 CLI/MCP/摘要/完整报告中保持一致 | **是** | `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs::cli_subprocess_and_mcp_handler_agree_on_the_same_outcome`(由 WI-781 扩展) | 为 CLI 一侧启动真实 `ai-cockpit` 二进制子进程,为 MCP 一侧在进程内调用生产处理器,针对同一受控仓库与 Work Item，机器 `OutcomeV2` 做精确相等比较；英文、简体中文和日文的人类交接则比较原因、未知项、授权和后果语义，而不要求自然语言逐字相等。 |
| 6 | 更换语言不改变事实/授权范围/后果 | **是** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`;`crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | CLI 测试依次以 `AI_COCKPIT_LANGUAGE=en/zh-CN/ja` 启动真实二进制,断言机器可读的 stdout JSON 字段在各语言下逐字节相同,只有面向人的 stderr 文本本地化。对抗性语料测试进一步检查每个语义案例在每种语言下的 5 种措辞变体评估结果相同。 |
| 7 | 展示的下一步与当前 Runtime 状态/策略一致 | **是(有界)** | `crates/cockpit-repository/tests/scenario_matrix_next_action.rs`(WI-741、PR #713); `crates/cockpit-cli/tests/collaboration_consistency.rs::displayed_option_state_and_runtime_transition_stay_consistent_through_resume`(WI-753) | WI-741 将展示的下一步消息绑定到实际观测的场景矩阵。WI-753 进一步断言展示的人工决定选项与 checkpoint、中断、恢复过程中的每个 Runtime 转换一致。覆盖是有界的,不构成所有状态的通用判定器。 |
| 8 | 既有授权是否适用由规则与记录决定,不因会话切换而改变 | **是(有界)** | `crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse`; `crates/cockpit-cli/tests/collaboration_handoff.rs::new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history`(由 WI-781 扩展) | preflight 测试证明只有在身份绑定与快照匹配时才能复用决定,过期复用会被拒绝。交接重建现在从声明授权加上 preflight、最新证据和治理控制记录推导 `applicable`,因此只读取 authority 字段不能证明适用性。覆盖不模拟所有可能的调用者身份转换。 |
| 9 | 每个需要人工决定的问题都指明对象/影响/恢复条件 | **是** | `crates/cockpit-repository/tests/contract_preflight.rs::assert_human_decision_request_is_complete`,由 `::scaffold_preflight_is_not_ready_and_records_human_review_requirements` 与 `::high_risk_scenario_coverage_stops_at_preflight_for_human_review` 调用(由 WI-710、PR #702 新增) | 断言 `what_happened`、`why_it_matters`、`question`、`resume_condition`、`options`、`recommended_option`、`recommendation_reason` 均非空,`recommended_option` 指向某个已提供的选项,且每个选项的 `id`/`label`/`effect` 均非空——针对两个独立触发的真实 `needs_human_confirmation` 场景进行检查。 |
| 10 | 摘要可以省略细节但不能隐藏阻断/关键未知项/必要决定 | **是** | `crates/cockpit-repository/src/outcome_render.rs::tests::human_renderer_preserves_blockers_and_unknowns_in_summary`; `crates/cockpit-repository/tests/outcome_report.rs::default_summary_is_four_part_and_full_view_retains_audit_sections` | 直接断言已填充的验收/意图阻断、失败 gate、恢复条件、未声明收益和人工下一步仍保留在摘要中；集成测试同时验证四段摘要和完整审计视图。 |

## 如何阅读本表

- **是**表示本仓库自身测试套件中已存在一个通过的测试,确实按不变量所述强制执行,本页引用了确切的函数。
- **部分**表示存在真实覆盖,但未触及不变量所述的全部情形(见该行说明)。
- **间接**(仅不变量7)表示存在相关的状态转换断言,但没有测试直接、端到端地断言该具体主张。

## 剩余边界

不变量7不再是名义上的零覆盖缺口:WI-741 与 WI-753 已提供绑定实际观测
Runtime 状态的有界可执行检查。WI-781 将场景 ID 绑定到可执行检查注册表，增加
人类语义对齐、finalization 动作分类、有界 Outcome 组装检查和分阶段 Runtime
诊断。暂不支持的子进程计数保持 unavailable，而不是估算为零。这些检查都不是
覆盖所有状态、选项或调用者身份组合的通用判定器。
