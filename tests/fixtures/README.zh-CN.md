# 夹具布局

conformance case 包含 `input.json`、`contract.json`、`repository/`、`evidence/` 和
`expected.json`。Rust harness 从磁盘加载输入与期望语义。V1 provenance 固定在
`../conformance/v1-reference.lock`；普通测试不会调用 V1 runtime。
