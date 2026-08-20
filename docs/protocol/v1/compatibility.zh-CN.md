# Protocol 兼容规则

兼容算法保持很小：

1. 在不执行 repository material 的情况下解析 protocol version。
2. malformed 或不支持的 major version 直接 Red。
3. 只有所需 artifact 字段有效时，才接受支持的 major version。
4. 可选 capability 缺失报告为 Yellow，并给出明确 safe action。
5. 兼容性检查不能重写历史 artifact。

Runtime 宣布支持的 protocol range，repository 宣布一个 protocol major。Runtime 的
minor/patch release 不是 migration。Major protocol migration 必须创建新 Work Item，
保留旧 evidence，并记录 source/target protocol version。

