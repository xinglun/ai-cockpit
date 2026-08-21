# Conformance harness

每个 case 都包含 repository material、contract、evidence、明确的 governance input 和期望
语义结果。普通 Rust test 从磁盘加载文件并比较决策字段而不是格式；它是快速离线回归，
不下载或执行 V1。

Gate B 另有可执行边界。`v1-reference.lock` 固定精确的 V1 reference commit。专用
CI job 检出该提交、设置 `AI_COCKPIT_V1_ROOT`，并运行
`cargo test -p cockpit-core --test v1_oracle -- --ignored`。测试先验证 checkout
身份，然后由 `v1_oracle.py` 对十四个 fixture 调用 V1 治理 primitive，并比较
decision state、blockers、unknowns、safe actions、required checks、authority 和
outcome state。Adapter 从不读取 `expected.json`，所以 mismatch 是独立证据，
不是 Rust 结果的第二次投影。

外部 V1 Runtime 与 Python 只属于 conformance 测试依赖；不会链接进 Rust binary，
也不会附加到 adopter repository。
