# M2 Dialogue Playback

## 范围

本阶段实现最小 Dialogue 播放模型，让 engine 可以从版本化场景数据启动对话、展示选择、结算效果。

## 当前结构

- `DialogueScene`：场景 ID、入口节点和节点列表。
- `DialogueNode`：说话人、文本和可选选择。
- `DialogueChoice`：选择 ID、显示文本、下一节点和效果列表。
- `DialogueEffect`：当前支持角色状态调整、关系调整、天气切换和日志写入。
- `EngineCommand::ChooseDialogue`：前端只提交选择命令，状态由 engine 结算。

## 规则

- `StartDialogue` 只激活场景入口节点。
- 选择不存在、节点不活跃、未开始对话时命令失败且世界状态回滚。
- 有 `next_node_id` 的选择会追加下一节点；无下一节点的选择结束当前对话。
- 选择效果可更新角色状态和关系，但所有效果失败时整条选择命令回滚。
- Dialogue 数据不执行旧 ERB，不直接引用文件路径。

## 后续

- 增加条件判断、资源引用、占位符和变量类型校验。
- 将内容包加载器接入 `dialogue_scenes`。
- 为节点可达性和死链增加内容测试。
