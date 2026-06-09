# M2 Engine Scheduler

## 范围

本阶段建立最小事件调度器，用于驱动时间推进后的世界变化和确定性回放。

## 当前结构

- `ScheduledTime`：以 day/hour/minute 表达事件触发时间，序列化进入 `WorldState`。
- `ScheduledEvent`：稳定事件 ID、到期时间、优先级、可选循环规则、条件列表和事件类型。
- `ScheduledRepeat`：定义循环间隔和可选剩余触发次数。
- `ScheduledEventKind`：当前支持天气切换、对话启动、关系调整、角色状态调整。
- `EngineCommand::ScheduleEvent`：通过 command API 注册事件，前端不直接修改事件队列。
- `EngineCommand::CancelEvent`：通过 command API 取消仍在队列中的事件。
- `Relationship`：保存来源 ID、目标角色 ID、好感和信赖，进入 `WorldState` 与存档。
- `EngineCommand::AdjustRelationship`：通过 command API 调整关系，前端不直接改关系数组。
- `WorldState.command_log`：记录成功结算的命令，随存档序列化保存。
- `WorldState.random`：显式保存随机种子和游标，用于可重放的随机结算。
- `WorldState.command_log_initial_random`：记录第一条成功命令前的 RNG seed/cursor，确保非默认 seed 的随机命令也能独立重放。
- `EngineReplayLog` / `replay_command_log`：从初始世界、初始 RNG 和成功命令列表重放世界，用于存档诊断和故障复现。
- `EngineCommand::RollCharacterMood`：当前用于验证随机命令、状态结算和回放一致性。
- `DialogueEffect::RollCharacterState`：内容效果可用同一 RNG 对角色体力和心情做有界随机结算。
- `replay_commands`：从初始世界和命令列表重放出确定结果。
- 内容包安装后的 `ScheduledEventKind::StartDialogue` 可启动同包新增的 `DialogueScene`，由跨模块调度测试覆盖。

## 规则

- 事件 ID 不能为空，同一队列内不能重复。
- 无效时间会被拒绝。
- 循环事件间隔必须大于 0，`remaining_runs` 不能为 0。
- `apply_command` 使用事务式更新：命令失败时原 `WorldState` 不变。
- 只有成功命令进入 `command_log`，失败命令不污染回放历史。
- `advance_time` 触发所有到期事件；事件按到期分钟、优先级降序和 ID 排序。
- 到期事件只触发一次，触发后从队列移除。
- 到期但条件未满足的事件保留在队列中，后续时间推进继续尝试。
- 事件条件复用 `DialogueCondition`，当前支持地点、心情、关系、天气和时间判断。
- 循环事件只在成功触发后重排；条件未满足时不会消耗剩余次数。
- `remaining_runs` 表示该循环事件还可成功触发的总次数；`null/None` 表示无限循环。
- 一次大跨度时间推进会按循环间隔补触发所有已经到期的循环事件。
- 内容包事件安装后进入同一调度队列；触发包内对话时仍走 engine `StartDialogue` 路径。
- 取消不存在的事件会失败并回滚，不写入 `command_log`。
- 角色状态调整使用边界约束：体力 `0..100`，心情 `-100..100`。
- 关系调整使用边界约束：好感/信赖 `-100..100`。
- 关系目标必须是已存在角色；关系来源可为 `player` 等稳定领域 ID。
- 随机数不读取系统熵；所有随机结果由 `WorldState.random.seed + cursor` 派生。
- `seed` 和 `cursor` 在 JSON 中以字符串保存，避免前端 64 位整数精度损失。
- replay log 记录第一条成功命令之前的 `seed/cursor`，不要求调用方知道世界的原始启动 seed。
- 随机命令失败时不推进 `cursor`，成功后才进入 `command_log`。
- 随机范围必须满足 `min_delta <= max_delta`，否则命令整体回滚。
- Dialogue 随机效果使用 `WorldState.random`，非法范围或缺失角色会让整条选择回滚且不消费 RNG。
- 旧存档缺少 `random` 字段时使用默认 demo seed 迁移读取。

## 后续

- 将随机结算接入事件条件和更多互动命令。
- 将 replay log 接入故障反馈包、开发者诊断 UI 和长时间游玩回归样例。
