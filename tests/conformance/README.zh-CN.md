# Conformance harness

每个 case 都包含 repository material、contract、evidence、明确的 governance input 和期望
语义结果。Rust test 从磁盘加载这些文件，只比较决策字段，不比较格式。
`v1-reference.lock` 记录编写 corpus 时使用的 V1 reference commit；普通构建不会下载或执行 V1。
