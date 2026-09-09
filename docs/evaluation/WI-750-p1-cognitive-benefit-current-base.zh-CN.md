# WI-750：P1-A 认知收益评估

状态：评估材料已交付；认知收益尚未验证。

本 Work Item 为 **Calibrated Human-Agent Trust** 准备可重复的对比方法，
不宣称用户阅读更快、错误更少或风险拦截更多：本次没有真实参与者，也没有
读取或写入外部采用方或其他仓库的数据。

## 固定方法与答案键

任务集和答案键在比较前固定于
[`tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py`](../../tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py)。
脚本读取真实归档 `OutcomeV2` 与 `TaskOutcomeReport`，用当前源码构建的 CLI
分别生成 `--view summary` 和 `--view full`，比较：

- 验证、生命周期、人工决定和治理信号；
- 证据引用及四个摘要部分；
- 人的下一步和关键不确定性是否保留；以及
- 中、日、英章节文案。

完整报告是审计投影，摘要是默认阅读投影。自动调用完整报告只是对照检查，
不是用户阅读事件。原始证据只作为数据，不能成为指令或授权来源。

## 固定任务集

| 任务 | 真实 Outcome 来源 | 边界事实 | 评分答案 |
| --- | --- | --- | --- |
| 正常完成 | `WI-663-wi659-outcome-trust-replacement` | 已记录 `finish_ready/green` | 绿色验证不是合并或发布授权，仍需查看证据和风险边界。 |
| 验证通过、待人工决定 | `WI-658-wi656-outcome-trust-repair` | 已记录 `finish_ready/green`，没有 close 决定 | 人工决定未记录；审阅证据并记录明确的 close 决定。 |
| 范围超出 | `WI-714-wi713-current-base-revalidation` | `tests/conformance/fixtures/scope-exceeded/input.json` | 停止；范围是边界，不授权继续。 |
| 证据过期或身份不匹配 | `WI-423-ci-convergence` | `tests/conformance/fixtures/contradictory-evidence/input.json` | 停止并获取新的、身份匹配的证据。 |
| 测试弱化信号 | `WI-662-p0-benchmark-evidence` | `tests/conformance/fixtures/test-weakening/input.json` | 检查范围和扫描结果；空风险记录不等于“没有弱化”。 |
| 未验证范围 | `WI-139A-preflight-review` | Contract 记录了未验证场景和验证计划 | 保留未验证范围，在宣称完成前完成计划。 |
| 历史已关闭任务 | `WI-743-wi715-p1-current-base-redelivery` | 已关闭决定与历史证据 | 保留历史；只有需要当前结果时才重新验证，不提升保证级别。 |

归档源状态和当前 Runtime 投影都保存在生成证据中。历史或 foreign Runtime
投影显示为历史、未知或尚未就绪，是信任边界，不会被静默评分为当前绿色成功。

## 结果与限制

结果记录在 `.ai/evidence/WI-750-p1-cognitive-benefit-current-base.json`，阅读版
摘要在 `.ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md`。

当前运行记录：7 个固定任务、0 个摘要/完整报告一致性错误、0 个关键可见性错误、
0 名参与者，认知收益 0 个已验证。正确识别时间、错误放行、关键风险遗漏、把绿色
理解为安全或授权、以及参与者查看完整证据的次数仍为“未测量”。完整报告的 14 次
调用全部是自动一致性检查，不是用户行为。

重复执行：

```sh
bash tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh
```
