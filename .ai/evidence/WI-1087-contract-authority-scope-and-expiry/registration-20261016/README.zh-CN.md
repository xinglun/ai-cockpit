# WI-1087 planned 设计登记

此提交只保存 Ray 经 Raydot 传达的设计和 planned 登记请求。现有 Runtime 没有 `planned` 生命周期枚举；实际使用正常 `new --mode planning`，状态为 `not_ready`、checkpointCount=0、authority=missing。没有启动实现、preflight、checkpoint、产品构建或验证。

通过 typed amend 填充设计范围；正常 declare 声明依赖且冲突于 WI-1086，parallelizable=false。WI-1086 不在本 checkout 的 main 清单中，不能据此伪造依赖完成或实施准入。产品代码、共享 parity 和既有授权均未修改。

设计以 `../design-20261016/design-v1.zh-CN.md` 为唯一详述，另有 23 项 not_run 验收和未来实施阶段；三语工作项简报引用同一材料。未来实施需正常准入及技术审阅，本登记不配置真实 grant 或修改既有权限。

原始 stdout/stderr/meta 原样保留。status/outcome 中的 authority/user-benefit、未验证、未审阅等未知保留，不以规划材料冒充通过。共享 parity 行在当前范围外，未添加。
