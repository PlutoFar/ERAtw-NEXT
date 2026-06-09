# M2 Dialogue Playback

## 范围

本阶段实现最小 Dialogue 播放模型，让 engine 可以从版本化场景数据启动对话、展示选择、结算效果。

## 当前结构

- `DialogueScene`：场景 ID、入口节点和节点列表。
- `DialogueNode`：说话人、文本、资源引用和可选选择。
- `DialogueChoice`：选择 ID、显示文本、下一节点和效果列表。
- `DialogueCondition`：当前支持地点、角色心情、关系好感、天气和时间判断。
- `DialogueEffect`：当前支持角色状态调整、关系调整、天气切换和日志写入。
- `EngineCommand::ChooseDialogue`：前端只提交选择命令，状态由 engine 结算。
- `eratw_content::ContentPackage`：封装 manifest、Location、Character、Relationship、ResourceAsset、DialogueScene 与 ScheduledEvent 列表。
- `ContentPackage::validate`：在运行前报告角色/地点/关系结构错误、资源元数据缺失、入口缺失、死链、重复 ID、不可达节点、非法调度事件等问题。
- `ContentPackage::install_into_world`：只安装校验干净、满足内容包依赖/冲突声明，且不与现有世界 ID 冲突的 Location/Character/Relationship/ResourceAsset/DialogueScene/ScheduledEvent。
- `engine_install_content_package`：桌面端通过 Tauri command 安装内容包，前端和浏览器 mock 共用同一安装语义。

## 规则

- `StartDialogue` 只激活场景入口节点。
- 选择不存在、节点不活跃、未开始对话时命令失败且世界状态回滚。
- 有 `next_node_id` 的选择会追加下一节点；无下一节点的选择结束当前对话。
- 选择条件不满足时不会在 UI 展示；强行提交该选择会被 engine 拒绝并回滚。
- 选择效果可更新角色状态和关系，但所有效果失败时整条选择命令回滚。
- Dialogue 数据不执行旧 ERB，只引用 `resourceId`，不直接引用文件路径。
- ResourceAsset 的 `source_path` 只能是相对安全路径；运行时通过资源解析报告提供 planned/missing/hash_mismatch 等状态和降级 fallback。
- 内容包新增角色必须引用已存在或同包新增的地点；关系、条件、效果和事件动作必须引用已存在或同包新增的角色/关系。
- 内容包校验返回结构化 issue code 和 target，供 CLI、编辑器和运行前检查复用。
- 内容包必需依赖缺失、依赖版本不匹配、正向或反向冲突声明命中时，安装失败且世界不变。
- 内容包安装失败时返回错误，不修改输入 `WorldState`。
- 内容包事件可启动同包新增的 DialogueScene，安装后由 engine scheduler 正常触发。

## 后续

- 增加占位符和变量类型校验。
- 为条件、资源引用覆盖和占位符增加更完整内容测试。
