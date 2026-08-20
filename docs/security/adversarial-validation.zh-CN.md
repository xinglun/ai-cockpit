# 对抗性验证

安全边界采用 fail-closed 和证据驱动原则。conformance corpus 比较语义而不是字符串：
决策状态、阻断项、未知项、安全动作、必需检查、权限和结果状态必须一致。

运行时边界测试还验证仓库文本只作为数据、Work Item ID 不能路径穿越、MCP evidence 路径
必须位于仓库内、验证命令使用 allowlist 和目标 cwd，以及 finish 不能在没有新鲜通过回执时
自我声明完成。

失败或未知的 provider 结果始终不是 green。人类权限可以解决决策要求，但不能伪造验证回执。
