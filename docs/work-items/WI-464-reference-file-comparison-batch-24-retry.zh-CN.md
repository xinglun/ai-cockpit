# WI-464 参考源文件比对第 24 批——恢复重试

## 意图

保留 WI-464 的不可变交付尝试，并在真实 Provider context 下完成同一批受限比对。本恢复 Work Item 不扩展源到目标的范围，也不复制参考实现字节。

## 来源与边界

- 参考工程：`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`
- 固定源提交：`fde3380f81fea5fd2e288f7a8849f737dc074060`
- 前置 Work Item：`WI-464-reference-file-comparison-batch-24`
- 恢复原因：前置 Work Item 在真实 Provider PR 存在前绑定了占位 PR URL；其证据保持不可变，不重写。

## 比对路径

| 参考路径 | Rust 侧结果 |
| --- | --- |
| `.github/workflows/compatibility.yml` | 按设计实现不同；Rust CI 使用自身的 action 固定版本和平台策略。 |
| `.github/workflows/release.yml` | 按设计实现不同；Rust release manifest、SBOM、provenance、校验和与 adopter harness 提供发布边界。 |
| `.github/workflows/smoke.yml` | 按设计实现不同；Rust 生命周期及 release/adopter 检查替代源 Make bridge。 |
| `Makefile` | 按设计实现不同；支持的接口是 Rust CLI、Cargo 检查和 repository gate manifest。 |

未发现 Rust 遗漏。仅源侧 Python/Make/installer 行为明确不在范围内。

## 交付规则

必须先创建真实且可审查的 PR，再由 `finalize-plan` 绑定 URL。随后按顺序执行 preflight、checkpoint、verification、finish、archive、finalization 和 close。前置恢复 receipt 与本次证据均保持追加式、绑定本仓库。

## 验证

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060
python3 tests/conformance/reference_inventory_docs_test.py
```
