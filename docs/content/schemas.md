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

## 规则

- 所有 ID 必须稳定、可迁移、带命名空间。
- 剧情只引用 `resourceId`，不直接引用文件路径。
- 旧 ERB 只能生成参考草稿和审计线索，不进入运行内容包。
- `license` 与 `author` 在发布前不得保持 `unknown`。
