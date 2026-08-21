# リリースと配布の evidence

リリース workflow は macOS arm64、macOS x86_64、Linux arm64、Linux x86_64、Windows x86_64 向けに
単一の `ai-cockpit` binary を構築します。各 artifact には SHA-256 checksum、Cargo metadata、
SPDX SBOM、GitHub build-provenance attestation を添付します。Unix target は `.tar.gz`、Windows は `.zip` に梱包します。

checksum と metadata は release evidence であり、governance core が自己証明するものでは
ありません。保護された `COSIGN_*` secret を設定した場合、workflow が checksum を署名します。
本番の鍵管理と release 環境の承認は保護された human/CI control です。GA gate を green とする
前に receipt を添付してください。
