# M2 Engine Scheduler

## 范围

本阶段建立最小事件调度器，用于驱动时间推进后的世界变化和确定性回放。

## 当前结构

- `ScheduledTime`：以 day/hour/minute 表达事件触发时间，序列化进入 `WorldState`。
- `ScheduledEvent`：稳定事件 ID、到期时间和事件类型。
- `ScheduledEventKind`：当前支持天气切换、对话启动、角色状态调整。
- `EngineCommand::ScheduleEvent`：通过 command API 注册事件，前端不直接修改事件队列。
- `WorldState.command_log`：记录成功结算的命令，随存档序列化保存。
- `WorldState.random`：显式保存随机种子和游标，用于可重放的随机结算。
- `EngineCommand::RollCharacterMood`：当前用于验证随机命令、状态结算和回放一致性。
- `replay_commands`：从初始世界和命令列表重放出确定结果。

## 规则

- 事件 ID 不能为空，同一队列内不能重复。
- 无效时间会被拒绝。
- `apply_command` 使用事务式更新：命令失败时原 `WorldState` 不变。
- 只有成功命令进入 `command_log`，失败命令不污染回放历史。
- `advance_time` 触发所有到期事件；事件按到期分钟和 ID 排序。
- 到期事件只触发一次，触发后从队列移除。
- 角色状态调整使用边界约束：体力 `0..100`，心情 `-100..100`。
- 随机数不读取系统熵；所有随机结果由 `WorldState.random.seed + cursor` 派生。
- `seed` 和 `cursor` 在 JSON 中以字符串保存，避免前端 64 位整数精度损失。
- 随机命令失败时不推进 `cursor`，成功后才进入 `command_log`。
- 随机范围必须满足 `min_delta <= max_delta`，否则命令整体回滚。
- 旧存档缺少 `random` 字段时使用默认 demo seed 迁移读取。

## 后续

- 增加事件条件、优先级、取消、循环事件。
- 将随机结算接入事件条件、互动命令和内容效果。
- 将 Dialogue/Scene 内容包接入 `StartDialogue`。
- 补内容包加载后的跨模块调度测试。
