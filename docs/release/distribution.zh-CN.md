# 发布与分发证据

发布工作流为 macOS arm64、macOS x86_64、Linux x86_64 和 Windows x86_64 构建单一
`ai-cockpit` binary。每个产物附带 SHA-256 校验和，以及作为 SBOM 输入的 Cargo metadata。

校验和与 metadata 是发布证据，治理 core 不会自我证明它们。生产签名、密钥保管、provenance
attestation 和发布环境审批仍属于受保护的人类/CI 控制；在 GA 门禁变绿前必须附加这些回执。
