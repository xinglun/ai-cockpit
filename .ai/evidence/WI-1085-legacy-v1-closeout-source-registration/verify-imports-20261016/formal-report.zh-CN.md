# WI-1085 显式导入修复后的完整原生验证

Outcome: 🟡 正式验证已实际13/13通过；场景/意图与用户收益仍分别评估，不自动宣称整体验收。

源码HEAD=5c0212c6083abbdc6843c2e2647a6de283232e3d，实现源码提交284e31bed828c7cd04bb41261f5059ad8f1d6e69。当前Contract=sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1，repositorySnapshot=sha256:bddd1a809e3b04e357272e17cd1771cb80fb312f16301bd33f688ea4522a535e，Runtime=sha256:0ffdcd20740f22b96621cf06adb5479d83fb435f503a26c18d824d61c4bfe9c4。Raydot真实材料决定已原生记录为b8da3c27…，只绑定ce6cf9c9…与原两许可syntax Unknown，没有复用前次9dc8bd00…收据。

## 实际结果

成功attempt=sha256:e749e39f2d3bce872b6bb6e20a912ea3c72d549e38e31f3d2817c78eecf3aec2；正式canonical receiptDigest=sha256:92989aac97929788723d885157bd1767ae39e7b31a2dd93bf4403409d81eecee。两份原生receipt逐项完全一致，repository/WI/Contract/snapshot/Runtime绑定相同。13条execution record均spawned=true、passed=true、exitCode=0、timedOut=false；13实际执行、13通过、0失败、0复用、0超时，最大项目进程并发1，elapsedMs=842538，executionElapsedMs=840604。创建时间2026-10-16T14:05:14Z（JST23:05:14）。外围工具、timeout/Tini正常返回0，未触发全局超时；不声称强制超时回收路径经过本次实测。

previous status_projection与resource_finalization测试目标在原生stderr明确实际Running，repository整个节点通过；未重现signal7、磁盘os error28或显式导入守卫失败。旧失败保留，不由成功记录覆盖，不对先前Bus error作因果结论。

## 预热、focused与正式验证分开

原失败守卫focused1/1与锁模块focused3/3实际通过；随后同target/profile的13条--no-run命令exit0，但执行断言数为0。正式验证是之后单独调用Runtime，全部13节点仍实际执行，未把预热、历史或定向结果当作正式收据。正式启动/计划/事实/环境、原始日志及focused和prewarm回收证据在同目录。

## 资源与回收

同一隔离target、build jobs1、test threads1、workers1，Tini -s -g -vvv -w +890秒timeout及10秒kill余量；package上限600秒。启动可用6,471,835,648字节，结束6,446,010,368字节；所有完整过程快照高于2GiB预留线。高内存主要为可回收file cache；本轮新增OOM kill=0。没有额外清理、改profile、放宽断言或增加上限。

正式基线与结束僵尸均1431，PID+startTimeTicks无新增/丢失；起始live转Z新增0。Tini根1165975消失、同session与采样owned集合无live/zombie残留，无cargo/rustc活动，原始日志记录1528次孤儿僵尸回收、main child正常返回0及No child to wait。前后/过程枚举无未解决错误；竞态仅经确认PID消失才记resolved。采样不能枚举每个历史后代，限制保留。

## 字节、平台与验收限制

26份captured stream逐字节解码到短路径streams/，manifest记录长度、SHA256、truncated与实际原生字段历史引用。repository stdout仍为原生上限截断；其他stdout及所有stderr未截断，不补造缺失尾部。正式canonical/attempt均Runtime生成，未手改。

执行环境是Linux；未声称执行Windows/reparse或hosted CI。只读查询当前测试HEAD已有GitHub checks为0，保留原始结果existing-head-checks.json。跨平台CI属于后续集成证据，不用本地通过替代。

11个场景、17项验收映射及意图需分别用当前真实输出/源码/文档核对，再按原生controls记录；收益如无真实用户观察保留Unknown/推断。此验证结果提交不擅自创建PR、转Ready、merge、archive、close或清理，也不启动WI1087。
