# 跨 Work Item 协调

此能力只由候选 Runtime 提供。Runtime `0.2.105` 继续负责生命周期，不读取
协调记录。候选 Runtime 负责能力发现、仓库本地协调读写、影响准入和精确组合
验证；这不是双向兼容承诺。

## 支持边界

协调根目录由 Git 的 common directory 解析，因此同一仓库的 linked worktree
可以协作。独立 clone 和跨机器协调不支持。记录位于
`.ai-cockpit/coordination/v1/`，并绑定仓库、WI、Contract、worktree/head、候选
Runtime 能力和执行代次。

普通单 WI 生命周期仍是快速路径：除非 WI 明确参与协作，否则不扫描、不写入
协调存储。

只读查询：

```text
ai-cockpit work-item coordination inspect --repo <path>
```

登记、影响报告、暂停请求/确认、恢复消费和组合验证都是显式写入入口：

```text
ai-cockpit work-item coordination register --repo <path> --input registration.json
ai-cockpit work-item coordination report-impact --repo <path> --input event.json
ai-cockpit work-item coordination request-pause --repo <path> --input request.json
ai-cockpit work-item coordination acknowledge --repo <path> --request-id <id> --state acknowledged
ai-cockpit work-item coordination resume --repo <path> --id <wi> --generation <n>
ai-cockpit work-item coordination recover --repo <path> --event-id <id> --consumer-work-item-id <wi> --consumer-generation <n>
ai-cockpit work-item composition --repo <path> --id <wi> --generation <n> --input composition.json
```

查询不会修复、消费、确认或创建目录。恢复消费按事件、提供方代次和消费者
代次幂等；旧代次会被拒绝。影响事件按身份去重，只阻塞受影响消费者，无关 WI
继续执行。暂停请求、确认、安全暂停、不可用/过期和恢复是不同状态。

组合验证会先刷新依赖准入，再在临时 linked worktree 中按声明顺序组合。前置
条件失败时昂贵验证进程数为零；只有所有源、依赖、接口、配置、toolchain、锁
文件、生成输入、环境、验证器和命令身份都一致且前次成功，才允许复用证据。

CLI 与 MCP 调用同一仓库服务。Outcome 将实现、组合、目标合并和清理分别展示；
没有证明的收益或缺少可比基线会保持为未知，不会被写成性能结论。
