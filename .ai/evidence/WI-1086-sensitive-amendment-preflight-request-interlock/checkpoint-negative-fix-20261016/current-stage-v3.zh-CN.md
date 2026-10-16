# 当前回归结果与交接

源码 2a5910e967585c3589af382d29143f1b50b0a2c1：已准入定向 RED 10/10 后实施最小修复，当前 CLI/MCP28、CLI scope fixture1、repository lifecycle52、repository lib126，共207项通过，格式检查通过。每个测试启动前由同一执行器读取实际 Runtime admission，全部为 allowed。单构建任务、单测试线程、原隔离 target、900秒外层上限和256MiB磁盘保护均有原始记录。CLI 构建产生的实际候选 SHA-256 在 JSON 中；后续 cargo --lib/fmt 不声称生成了新 CLI。

两个 checkpoint 来源都绑定首个原生 checkpointed transition 的 Contract 前驱，避免 null/null 和成对伪造；fallback 使用原生 checkpointed 事实而非可篡改 Summary count。双 amendment 原生正例及历史连续性负例仍通过。三处 fixture 采用原完整 preflight/断言块前移，保留所有原 assert。

真实 Ray 确认已记录为追加 receipt（3f1f7cec…），Contract 保持e7245a06…，源码变化后仅 Runtime 当前观察更新，真人 receipt 不改绑、不重复造批准。旧真人 receipt 和原完整失败 attempt 原样保留。

必须区分：207项定向通过不替代 Contract 声明的 workspace 完整正式回执；本轮未重跑正式 workspace 或观察新head CI。Runtime 当前仍 checkpointed / verification not_ready，意图对齐、用户收益/正式验证等缺口如实保留；不执行最终验收、finish/archive、Ready/merge/close 或共享Runtime安装。

清理核验279文件全匹配，但进程FD占用观察存在errno13，因此按明确跳过条件全部保留，实际释放0。当前空间仍不足以默认完整重建安全。旧未准入7.663秒观察单独保留；执行器错误及失败即停止修正记录完整，没有将该轮当正式验证。
