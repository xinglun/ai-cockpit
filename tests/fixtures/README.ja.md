# フィクスチャ構成

conformance case は `input.json`、`contract.json`、`repository/`、`evidence/`、
`expected.json` を含みます。Rust harness は入力と期待する意味をディスクから読み込みます。
V1 provenance は `../conformance/v1-reference.lock` に固定し、通常テストでは V1 runtime を呼びません。
