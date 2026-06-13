# ERAtw-NEXT M4 玩法状态机与存档基础设计

## 结论

M4 建立不依赖 ERB 的确定性玩法状态机、世界时间、事件队列、命令 reducer、版本化存档 envelope 和 replay 校验。状态逻辑位于 engine，Tauri 只保存当前内容包与游戏会话，UI 只调度命令。

实现状态：已完成。

## GameState

`game-state/v1` 包含：

- package identity。
- turn。
- day、minute of day、total minutes。
- current location。
- player energy/max energy/money。
- integer flags。
- 按 `dueAt + id` 稳定排序的 event queue。
- 最近一次命令触发的 event records。

状态不包含系统时间、随机数、线程状态或 UI 状态。

## Command Reducer

`apply_command(context, state, command)` 是纯函数。支持：

- `wait`：推进时间。
- `rest`：推进时间并恢复体力。
- `move`：验证地点连接、扣除体力并推进时间。
- `setFlag`：写入整数 flag。
- `scheduleEvent`：加入稳定排序的事件队列。

所有成功命令增加一个 turn。时间推进后，所有到期事件按确定顺序出队。非法分钟、断开的地点、体力不足、重复事件 ID 和错误 state/package context 返回结构化错误。

`replay_commands` 从 initial state 顺序执行 command log。同一 initial state、content package 和命令序列必须得到相同最终状态。

## Session

`GameSession` 保存：

- 不可变 `GameContext`。
- initial state。
- current state。
- command log。

加载新内容包时桌面 runtime 清空 session。新游戏只允许在 `playable: true` 的包上创建。

## Save Envelope

`save-envelope/v1`：

```text
schemaVersion
saveVersion
engineVersion
dependencies[]
initialState
state
commandLog[]
stateHash
```

规则：

- 当前 save format version 为 1。
- dependency 记录 package ID 与精确版本。
- `stateHash` 是 state JSON 的 SHA-256。
- 保存路径必须是本地绝对 `.json` 路径。
- 不覆盖已有存档。
- 先写同目录临时文件、flush、sync，再 rename，避免产生半写入正式存档。
- 单存档上限 16 MiB。

## 加载与迁移占位

加载顺序：

1. 路径、普通文件和大小校验。
2. 严格 JSON 反序列化，拒绝未知字段。
3. schema/save version 校验。
4. engine version 校验；未来 engine 生成的存档拒绝加载。
5. package dependency ID/version 校验。
6. initial/current state package identity 与状态不变量校验。
7. state hash 校验。
8. 从 initial state replay command log。
9. replay state 与保存 state 完全相等后才替换当前 session。

状态不变量包括时钟字段一致、最大体力非零、flag key 合法、事件 ID 唯一、事件队列按 `dueAt + id` 排序且不包含已到期事件。时间与回合达到上限时返回结构化错误，不发生整数溢出或 panic。

目前没有旧格式，因此迁移表为空；未知 save version 返回 `SAVE_VERSION_UNSUPPORTED`，不会猜测迁移。

主要错误码：

- `SAVE_JSON_INVALID`
- `SAVE_VERSION_UNSUPPORTED`
- `SAVE_ENGINE_VERSION_NEWER`
- `SAVE_DEPENDENCY_MISSING`
- `SAVE_PACKAGE_MISMATCH`
- `SAVE_STATE_INVALID`
- `SAVE_HASH_MISMATCH`
- `SAVE_REPLAY_MISMATCH`
- `SAVE_ALREADY_EXISTS`
- `SAVE_PATH_UNSAFE`

## API 与 UI

Tauri commands：

```text
game_new() -> GameState
game_get_state() -> GameState | null
game_apply_command(command) -> CommandResult
save_write(path) -> SaveReport
save_load(path) -> GameState
```

游戏页展示时间、地点、回合、体力、金钱、事件数量，提供等待、休息和相邻地点移动。存档使用原生打开/保存对话框，并保留手工路径输入。

## 验收

- reducer 单元测试覆盖确定性 replay 与非法移动。
- 存档 round trip 保持 state 和 command log 完全一致。
- 错误 JSON、缺依赖、未来版本、hash/replay 不一致有明确错误。
- 仓库外自有包真实运行：等待 120 分钟、移动 10 分钟、到期 daybreak 事件、保存、读取、replay 成功。
- 真实结果：turn 2，day 1 08:10，地点 `core.location.square`。
