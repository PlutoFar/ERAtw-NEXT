# 内容 Schema 草案

M1 schema 只定义边界和审计目标，不承诺运行时完整实现。

## ContentPackageManifest

```json
{
  "schemaVersion": "content-package/v0",
  "namespace": "core",
  "packageId": "core.demo",
  "version": "0.1.0",
  "engineVersion": "0.1.0-m0",
  "dependencies": [],
  "conflicts": [],
  "migrations": []
}
```

## AssetManifest

```json
{
  "schemaVersion": "asset-manifest/v0",
  "sourceRoot": "D:/AICODE/ERAtw",
  "assets": [
    {
      "resourceId": "legacy.resources.100",
      "sourcePath": "resources/100.webp",
      "mediaType": "image",
      "sizeBytes": 12345,
      "sha256": "...",
      "license": "unknown",
      "author": "unknown",
      "usage": [],
      "characterBindings": [],
      "tags": []
    }
  ]
}
```

## DialogueScene

```json
{
  "schemaVersion": "dialogue/v0",
  "id": "core.demo.morning",
  "entryNodeId": "core.demo.morning.001",
  "nodes": [
    {
      "id": "core.demo.morning.001",
      "speakerId": "core.character.demo_heroine",
      "text": "早上好。",
      "choices": [
        {
          "id": "ask_about_engine",
          "label": "询问新引擎",
          "nextNodeId": "core.demo.morning.002",
          "conditions": [],
          "effects": [
            {
              "type": "add_log",
              "message": "对话选择：询问新引擎。"
            }
          ]
        }
      ],
      "conditions": [],
      "resourceRefs": []
    }
  ]
}
```

## ContentPackage

运行时内容包由 manifest 和若干已校验内容对象组成。当前 Rust `eratw_content`
MVP 已接入 DialogueScene 与 ScheduledEvent，后续再扩展角色、地点和资源。

```json
{
  "manifest": {
    "schema_version": "content-package/v0",
    "namespace": "core",
    "package_id": "core.demo",
    "version": "0.1.0",
    "dependencies": []
  },
  "dialogue_scenes": [],
  "scheduled_events": [
    {
      "id": "core.demo.morning_weather",
      "due": {
        "day": 1,
        "hour": 8,
        "minute": 30
      },
      "priority": 0,
      "repeat": null,
      "conditions": [],
      "kind": {
        "type": "change_weather",
        "weather": "cloudy"
      }
    }
  ]
}
```

## 校验报告

`ContentPackage::validate` 返回结构化 `ContentValidationReport`，用于 CLI、编辑器和
运行前检查复用。当前 issue code 覆盖：

- manifest 空 namespace/package_id。
- DialogueScene 空 ID、重复场景 ID。
- DialogueNode 空 ID、重复节点 ID、空文本。
- 入口节点不存在。
- Choice 指向不存在的下一节点。
- Choice 条件引用空 ID 或非法时间。
- 从入口节点不可达的节点。
- ScheduledEvent 空 ID、重复 ID、非法触发时间。
- ScheduledEvent 循环间隔非法或剩余触发次数为 0。
- ScheduledEvent 条件引用空 ID 或非法时间。
- ScheduledEvent 动作引用空角色/关系/场景 ID。

`ContentPackage::install_into_world` 只接受校验干净的包，并拒绝与当前
`WorldState.dialogue_scenes` / `WorldState.scheduled_events` 已有 ID 冲突的内容。
事件若启动对话，其 `scene_id` 必须能在安装后的世界中找到；失败时不修改输入世界。

## 规则

- 所有 ID 必须稳定、可迁移、带命名空间。
- 剧情只引用 `resourceId`，不直接引用文件路径。
- 旧 ERB 只能生成参考草稿和审计线索，不进入运行内容包。
- `license` 与 `author` 在发布前不得保持 `unknown`。
- 进入运行时前，内容包必须通过结构化校验报告，编辑器应直接定位 issue target。
