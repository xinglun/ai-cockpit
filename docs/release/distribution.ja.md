# リリースと配布の evidence

リリース workflow は macOS arm64、macOS x86_64、Linux x86_64、Windows x86_64 向けに
単一の `ai-cockpit` binary を構築します。各 artifact には SHA-256 checksum と SBOM 入力に
使う Cargo metadata を添付します。

checksum と metadata は release evidence であり、governance core が自己証明するものでは
ありません。本番署名、鍵管理、provenance attestation、release 環境の承認は保護された
human/CI control として残ります。GA gate を green とする前に receipt を添付してください。
