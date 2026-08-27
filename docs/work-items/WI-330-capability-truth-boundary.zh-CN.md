---
workItemId: WI-330-capability-truth-boundary
status: in_progress
---

# WI-330——能力真相边界校准

## 意图

将固定源版本中能力声明、freshness 与 Capability Truth Matrix 的四个文件逐个与 Rust
repository 对比，并为每个文件记录明确的产品边界。

## 逐文件决定

| 固定源路径 | 分类 | Rust 对应与决定 |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | 不复制 lexical claim-binding checker。目标 capability 页面和 registry 只报告有界的观察事实；文档 metadata 不是证据。 |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | 已验证 Work Item receipt freshness，但源 Capability Truth 行过期和 portable-environment policy 不是当前 Runtime 功能。 |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | 不把源三十行 matrix 作为 Rust wire format 或授权源。`capability_truth_registry` 按请求报告观察事实和明确排除。 |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | 目标 capability/adoption 文档说明 observed、repository、adopter、provider 和 enterprise 边界，不宣传源 matrix/checker。 |

## 边界

本 Work Item 关闭的是对比边界，不新增 claim checker、行级过期策略或 assurance 等级，也不复制
Python/V1 runtime 资产。未来 Rust 原生 claim/evidence 功能必须由单独的人工拥有 Work Item 定义
schema、过期处理、三语 scope 和 adopter 验收。

## 验收

- 每个固定源路径都有分类、对应物和不夸大能力的理由。
- 英文、简体中文和日文对比/parity 记录一致。
- 库存回归覆盖四个路径且计数不变。
- 不复制源 Python 脚本、源 matrix JSON 或 V1 runtime state。

## 验证

`bash tests/conformance/reference_file_inventory_test.sh`

