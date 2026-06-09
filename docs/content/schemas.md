# 内容 Schema 草案

M1 schema 定义边界和审计目标；其中 ContentPackage 的核心对象已经进入运行时
MVP，其余内容仍按草案推进。

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
MVP 已接入 Location、Character、Relationship、ResourceAsset、DialogueScene 与
ScheduledEvent。

```json
{
  "manifest": {
    "schema_version": "content-package/v0",
    "namespace": "core",
    "package_id": "core.demo",
    "version": "0.1.0",
    "dependencies": []
  },
  "locations": [
    {
      "id": "core.place.club_room",
      "name": "社团室",
      "ascii_symbol": "部",
      "terrain": "interior"
    }
  ],
  "characters": [
    {
      "id": "core.character.demo_heroine",
      "display_name": "示例角色",
      "location_id": "core.place.club_room",
      "state": {
        "energy": 80,
        "mood": 10
      }
    }
  ],
  "relationships": [
    {
      "source_character_id": "player",
      "target_character_id": "core.character.demo_heroine",
      "affinity": 5,
      "trust": 0
    }
  ],
  "resources": [
    {
      "resource_id": "core.demo.heroine.neutral",
      "source_path": "assets/demo/heroine-neutral.webp",
      "media_type": "image",
      "license": "project-demo",
      "author": "ERAtw-NEXT",
      "usage": ["portrait"],
      "character_bindings": ["core.character.demo_heroine"],
      "tags": ["neutral"],
      "sha256": null
    }
  ],
  "dialogue_scenes": [
    {
      "id": "core.demo.morning",
      "entry_node_id": "core.demo.morning.001",
      "nodes": [
        {
          "id": "core.demo.morning.001",
          "speaker_id": "core.character.demo_heroine",
          "text": "早上好。",
          "resource_refs": ["core.demo.heroine.neutral"],
          "choices": []
        }
      ]
    }
  ],
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
- Location 空 ID、重复 ID、空名称、空地形。
- Character 空 ID、重复 ID、空显示名、空位置。
- Relationship 空双方引用、重复关系。
- ResourceAsset 空 ID、重复 ID、空 source_path、空/unknown license、空/unknown author。
- DialogueScene 空 ID、重复场景 ID。
- DialogueNode 空 ID、重复节点 ID、空文本。
- DialogueNode 空资源引用。
- 入口节点不存在。
- Choice 指向不存在的下一节点。
- Choice 条件引用空 ID 或非法时间。
- 从入口节点不可达的节点。
- ScheduledEvent 空 ID、重复 ID、非法触发时间。
- ScheduledEvent 循环间隔非法或剩余触发次数为 0。
- ScheduledEvent 条件引用空 ID 或非法时间。
- ScheduledEvent 动作引用空角色/关系/场景 ID。

`ContentPackage::install_into_world` 只接受校验干净的包，并拒绝与当前
`WorldState.locations` / `WorldState.characters` / `WorldState.relationships` /
`WorldState.resources` / `WorldState.dialogue_scenes` / `WorldState.scheduled_events`
已有 ID 冲突的内容。Character 的位置、Relationship 的双方、DialogueNode 的说话人
和 `resource_refs`、Choice 条件/效果引用、ScheduledEvent 条件/动作引用，都必须能在
安装后的世界中找到；
事件若启动对话，其 `scene_id` 必须能在安装后的世界中找到；失败时不修改输入世界。
安装成功后，`WorldState.installed_content_packages` 会记录 manifest 的 namespace、
package_id 和 version；`SaveEnvelope` 会据此生成存档 `mod_dependencies`，当前以
`package_id` 作为依赖 namespace。

## 规则

- 所有 ID 必须稳定、可迁移、带命名空间。
- 剧情只引用 `resourceId`，不直接引用文件路径。
- 旧 ERB 只能生成参考草稿和审计线索，不进入运行内容包。
- `license` 与 `author` 在发布前不得保持 `unknown`。
- 进入运行时前，内容包必须通过结构化校验报告，编辑器应直接定位 issue target。
