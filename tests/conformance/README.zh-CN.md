# Conformance harness

每个 case 都包含 repository material、contract、evidence、明确的 governance input 和期望
语义结果。普通 Rust test 从磁盘加载文件并比较决策字段而不是格式；它是快速离线回归，
不下载或执行 V1。

Gate B 现在分成两个明确边界。托管 CI 只运行已提交的离线语义语料，从不访问参考仓库。
后续逐文件比较使用 `AI_COCKPIT_REFERENCE_ROOT` 指定的本地 Git checkout，并由
`reference-source.lock` 固定 commit；checkout 必须干净且 HEAD 必须匹配锁文件。
`reference_source_policy.py` 在不 clone、不 fetch 的情况下执行检查。旧的可执行 V1 oracle
仍可由维护者在本地使用 `v1-reference.lock` 和精确的本地 checkout 运行，但不再是托管 CI
依赖。

本地比较时，将 `AI_COCKPIT_REFERENCE_ROOT` 指向维护中的 checkout，并运行：

```bash
python3 tests/conformance/reference_source_policy.py \
  --lock tests/conformance/reference-source.lock \
  --reference "$AI_COCKPIT_REFERENCE_ROOT"
```

旧 V1 Runtime 与 Python 只属于可选的本地 conformance 依赖；不会链接进 Rust binary，
也不会附加到 adopter repository。
