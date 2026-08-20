# 发布与分发证据

发布工作流为 macOS arm64、macOS x86_64、Linux x86_64 和 Windows x86_64 构建单一
`ai-cockpit` binary。每个产物附带 SHA-256 校验和、Cargo metadata、SPDX SBOM 和 GitHub
build-provenance attestation。

校验和与 metadata 是发布证据，治理 core 不会自我证明它们。配置受保护的 `COSIGN_*` secret
时工作流会签名校验和。生产密钥保管和发布环境审批仍属于受保护的人类/CI 控制；在 GA 门禁变绿
前必须附加这些回执。
