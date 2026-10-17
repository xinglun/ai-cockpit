# WI-1085 单一显式导入修复与新材料交接

Outcome: 🟡 真实完整验证12/13已推送；在原scope内仅修正测试模块导入，等待新材料审阅及新源码验证。

失败结果提交ce9c3697cd2d1acafd68978f753704850633ee7d，attempt88ebd79a…：13实际执行、12通过、1失败、0复用、0超时、872836ms，源ef0a8354。唯一失败为resource_finalization.rs:36的显式导入守卫。MCP两项31→33回归已实际通过；本轮未重现Bus error/磁盘满，不对旧signal7作因果结论。完整stream、truncation、Tini回收与原生失败收据在../reverify-20261016/；旧失败不覆盖。

## 最小实施

修复提交284e31bed828c7cd04bb41261f5059ad8f1d6e69：仅在已有scope与implementationPaths内的resource_lifecycle.rs，将#[cfg(test)] legacy_source_lock_tests的use super::*展开为三个super函数和三个原有外部名字；未改生产行为、所有测试体、断言、守卫或Contract。整文件预期字节替换与实际SHA相同，守卫文件SHA不变，静态禁用文本已移除。静态字节检查不代表Rust编译或测试通过；新源码没有执行任何测试。

## 绑定与当前准入

新request=sha256:ce6cf9c9070014665e70ff86e1bdb09291911b6352ef1857ee77163d7f11acf1，Contract仍7fea3f7d…；sourceSnapshot=sha256:09ae28b7ac5c7c7330373195b8e3be916a33987a0dabc7104a55d67f7a1890f7，manifest=sha256:2ad0a864dbd4df4db08eb40429dc79835637fa1a9740506a0824789132e42c8a。27条、0Finding，只有resource_lifecycle条目更新且仍clean；其余26条材料完全一致，两份原许可syntax Unknown的字节/hunk/blob也不变。新材料请求未记录决定，旧9dc8bd00收据不复用。详见material-review-handoff.zh-CN.md与原生material-request.json。

Native preflight在当前实际source下成功：human_decision_recorded、humanDecisionRequired=false、blockers=[]；repositorySnapshot=sha256:bddd1a809e3b04e357272e17cd1771cb80fb312f16301bd33f688ea4522a535e。Summary仅由Native生成更新，没有手改人审绑定或要求新的Contract人审。当前等待Raydot普通技术材料决定；未伪造批准。

## 后续验证资源与时限

结束原完整验证后空间约6.03GiB。旧“51个新变体重建”11.21GB启动阈值不能直接套用；本轮已经成功编译全部13个原生package profile，下一补丁不改Cargo图、toolchain、依赖或feature。基于当前最大可执行文件165,839,240和最大repository rlib159,858,276，预算2倍各文件+1GiB临时开销后向上取整为2GiB，再保留2GiB，估算启动阈值4GiB；观测可用6,472,921,088字节，满足估算。此为同图单job替换已有产物的操作估计，不是未来空间数学保证，实际执行前及过程中必须再核对。没有进一步清理授权。

上轮872.836秒接近890秒外层边界，其中repository编译57.68秒，另有MCP与verification编译。为保留余量，下一轮在真实新材料决定后，先以相同13个package的cargo test参数仅加--no-run做compile-only预热并独立记录；保持单build job、同隔离target和Tini/890+10秒回收，不冒充断言执行。随后重查容量/基线再正式verify，13节点实际执行、不能用预热或定向测试替代。

11场景、意图对齐、用户收益与完整验收仍未完成。本轮无PR/Ready/merge/finish/archive/close或额外清理。下一步：Raydot对ce6cf9c9…实际新材料决定→原生record→容量/准入→compile-only预热→正式验证→提交真实结果。
