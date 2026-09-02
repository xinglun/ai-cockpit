---
author: AI Cockpit maintainers
title: Schema 与记录权限
description: AI Cockpit 的 Rust-native 记录映射和校验边界。
audience:
  - adopter
  - contributor
  - maintainer
  - auditor
status: current
authority: translation
canonical: docs/reference/schemas.md
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - typed_record_schemas
---

# Schema 与记录权限

[English](schemas.md) · [简体中文](schemas.zh-CN.md) · [日本語](schemas.ja.md)

可执行的 Rust Protocol 和 repository validators 决定记录是否有效。文档和示例只解释边界，不授予权限。每个仓库绑定记录都会携带 repository identity，并在适用时绑定 Work Item 或 snapshot。

| 记录或界面 | Rust-native authority | 边界 |
| --- | --- | --- |
| Work Item Contract | `cockpit-protocol` typed Contract 与 repository validation | 不会推断人工 intent、scope、authority、acceptance 或 verification。 |
| Change Summary | `.ai/work-items/` 下由 Runtime 生成的 Summary | changed paths、checkpoint、preflight、acceptance evidence 和 cost facts 被推导或绑定；Summary 不能授权变更。 |
| Project Profile | `.ai/project.json` 与 profile policy | 检测事实和人工确认分离；candidate proposal 不会改变 baseline。 |
| Repository Protocol | `.ai/cockpit.toml`、`project.json` 和 attached identity | Runtime 没有持久化的 current repository 或全局 Work Item。 |
| Verification Evidence | `.ai/evidence/<work-item>.verification.json` | 校验 schema、Work Item、repository、snapshot、runtime、receipt 和 `passed`；文件存在本身不是证据。 |
| Checkpoint Evidence | Summary 中 typed `checkpointEvidence` | stage、顺序、hash、计数、amendment 和 resume freshness fail closed。 |
| Delegated Evidence | `evidence import` metadata 与原始字节 digest | Provider/enterprise assurance 仍由外部负责；导入字节会被展示和绑定，不会被伪造。 |
| Archive 与 decision | archive manifest、finalization receipt、close decision | 这是不可变历史和人工决定边界，不是可编辑的状态缓存。 |

参考源 schema map 通过以下责任级 projection 覆盖。源记录名称不要求重新创建同名文件或 wire format：

| 源责任 | Rust-native projection |
| --- | --- |
| Project Profile | `.ai/project.json` 与 profile policy/validation |
| Cockpit checks | Contract 声明的 verification、动态 quality route 和 gate manifest |
| Capability status | `docs/reference/` 下 capability/status projection 与 request-scoped `status` |
| Documentation context | `.ai/README.md`、`.ai/glossary.md` 与 documentation-integrity checks |
| Archive discovery | archive index/manifest 与不可变 digest 校验 |
| Work Item Intelligence Snapshot | typed intelligence records 以及 `status`/`diagnose` projection |
| External handoff | human Outcome renderer 与 repository-bound MCP/Agent adapter projection |
| Outcome 与 status | Runtime 投影（`work-item outcome`、`status`） | 派生视图不能授权 merge、release 或 approval。 |
| Audit export | `audit export` event bundle | 长期不可变保存由外部 SIEM/WORM/retention 系统负责。 |

## 严格性与兼容

当前 V2 记录会拒绝损坏的必需字段、不安全路径、重复 identity、typed schema 中的未知嵌套字段、过期 snapshot 和跨仓库 evidence。旧记录保持不可变；无法满足当前 identity 要求时会投影为 historical/unknown，不会被静默重写或原地升级。

Rust 记录与参考源的职责在语义上兼容，但不是直接 JSON-wire 或 Python module 兼容。参考源的 `.ai/project_profile.yaml`、`.ai/cockpit/checks.yaml`、生成 status 文件和专用 registry 只是比对材料，除非明确记录了 Rust-native counterpart。
