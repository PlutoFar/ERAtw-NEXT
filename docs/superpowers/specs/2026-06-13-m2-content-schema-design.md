# ERAtw-NEXT M2 内容契约与转换草案设计

## 结论

M2 定义新内容包契约（content package contract）和只生成草案的转换策略。它承接 M1 只读审计结果，但不实现运行时加载、不接 UI、不修改 M0 UI/engine 行为、不把 `eratw-content` 内容本体写入 `ERAtw-NEXT`。

M2 的重点不是一次性迁移全部内容，而是建立一套能持续扩展、能验证、能追溯来源、能明确列出未迁移字段的 schema 边界。

实现状态：已完成。转换器只消费 M1 结构化报告，在 engine 仓库外生成不可玩的 draft package，并执行 JSON Schema、唯一 ID、交叉引用、source trace 和未映射覆盖校验。

## 输入事实

来自 M1 只读元数据盘点：

- 排除 `.git/` 后，`eratw-content` 有 6586 个源文件，约 350.05 MB。
- `ERB/` 有 3766 文件，约 149.54 MB。
- `resources/` 有 1325 文件，约 125.76 MB。
- `CSV/` 有 189 文件，约 0.40 MB。
- 全局 `.erb` 文件数大于 `ERB/` 内 `.erb` 文件数，说明教程/README 等目录也包含 ERB 样例。
- `resources/` 下存在图片、CSV、TXT、XML，不能把整个目录都当二进制资产。
- 资源文件有明显数字前缀和变体后缀，初步可作为 legacy character/resource binding 候选。

## M2 目标

- 定义 `content-package/v1-draft` 的目录结构、manifest 和核心对象。
- 定义首批 schema：character、location、resource、dialogue、dictionary、migration report。
- 定义从 CSV/资源/ERB 文件名到新 schema 的最小转换草案规则。
- 定义验证规则：schema 校验、引用校验、source trace 校验、未映射字段报告。
- 明确哪些内容可以自动转、哪些只能生成人工复核项。

## 非目标

- 不读取 legacy ERB/CSV 正文；转换器只读取 M1 报告。
- 不生成可玩的正式内容包，只生成待人工复核的 draft package。
- 不读取或输出 ERB/CSV 正文。
- 不做 ERB 解释器。
- 不做 UI。
- 不接入 `apps/`、`crates/`、root `package.json`。
- 不把转换产物提交到 engine 仓库。

## 推荐路线

三种可选路线：

1. **CSV 优先**：先把 CSV 词典、变量、角色基础字段 schema 化。实现简单，但资源和口上无法形成闭环。
2. **资源优先**：先建立 resource manifest 和角色资源绑定。适合视觉验证，但玩法语义不足。
3. **包契约优先**：先定义 package manifest、source trace、character/resource/dialogue 的共同边界，再分批填充。

采用路线 3。理由：ERAtw 内容体量大且混合度高，先定包契约和追溯机制，后续 CSV/资源/ERB 转换都能落在同一验证框架下，避免各自生成孤立 JSON。

## 内容包目录

M2 草案包建议结构：

```text
content-package/
  manifest.json
  dictionaries/
    legacy-csv-dictionaries.json
  characters/
    characters.jsonl
  locations/
    locations.jsonl
  resources/
    resources.jsonl
  dialogue/
    dialogue-sources.jsonl
    dialogue-scenes.jsonl
  migration/
    source-files.jsonl
    unmapped-items.jsonl
    validation-report.json
```

M2 定义结构并提供草案生成器，但不在 engine 仓库保存转换产物。真实输出必须进入临时目录或独立内容包仓库。

## 通用约定

### ID

所有新 ID 使用稳定字符串：

```text
core.character.001
core.resource.001.face.default
core.dictionary.talent
core.dialogue_source.kojo.001.daily
```

规则：

- 保留 legacy numeric id，但不把裸数字作为新主键。
- 新 ID 小写，使用 `.` 分段。
- 无法确定语义时使用 `migration.pending.<hash>`，同时进入人工复核清单。

### SourceTrace

每个自动生成对象必须携带来源追溯：

```json
{
  "sourceTrace": {
    "sourceRootId": "eratw-content",
    "relativePath": "CSV/Talent.csv",
    "legacyId": "Talent",
    "lineRange": null,
    "contentHash": null,
    "conversion": {
      "tool": "eratw-next-migration-draft",
      "version": "0.1.0",
      "confidence": "high",
      "requiresReview": false
    }
  }
}
```

M2 不要求计算 hash，但字段必须预留。若后续报告包含 hash，hash 是生成产物，不默认入 engine 仓库。

### ReviewState

所有转换对象有复核状态：

```json
{
  "review": {
    "status": "generated",
    "notes": [],
    "blockingIssues": []
  }
}
```

状态枚举：

- `generated`
- `needs_review`
- `accepted`
- `rejected`
- `blocked`

## Manifest Schema

`manifest.json` 描述内容包身份、依赖、兼容范围和转换来源。

```json
{
  "schemaVersion": "content-package/v1-draft",
  "packageId": "eratw.core.draft",
  "displayName": "ERAtw Core Draft",
  "version": "0.1.0-draft",
  "engineVersion": ">=0.1.0",
  "source": {
    "kind": "legacy-eratw",
    "sourceRootId": "eratw-content",
    "sourceRootHint": "D:/AICODE/eratw-content",
    "generatedFromAudit": "content-audit-summary/v1"
  },
  "dependencies": [],
  "conflicts": [],
  "capabilities": [],
  "review": {
    "status": "generated",
    "notes": ["Draft package is not playable."]
  }
}
```

验证规则：

- `packageId` 必须全局唯一。
- `schemaVersion` 必须精确匹配。
- `sourceRootHint` 只是提示，不用于运行时强制读取。
- draft 包必须标明不可玩。

## Dictionary Schema

CSV 中大量文件更适合先进入字典层，而不是直接变成玩法字段。

首批候选：

- `Abl.csv`
- `Base.csv`
- `CFLAG.csv`
- `CSTR.csv`
- `Equip.csv`
- `exp.csv`
- `FLAG.csv`
- `Item.csv`
- `Palam.csv`
- `Str.csv`
- `Talent.csv`
- `TCVAR.csv`
- `Tequip.csv`
- `TFLAG.csv`
- `Train.csv`
- `VariableSize.csv`

字典条目：

```json
{
  "id": "core.dictionary.talent.001",
  "dictionaryId": "core.dictionary.talent",
  "legacyKey": "001",
  "displayName": "Legacy Talent 001",
  "aliases": [],
  "valueType": "flag",
  "category": "character_trait",
  "sourceTrace": {}
}
```

M2 不要求理解所有 CSV 语义。无法确定的字段进入 `category: "unknown"` 和 `requiresReview: true`。

## Character Schema

角色对象承载最小身份、legacy id、资源绑定和字典引用。

```json
{
  "id": "core.character.001",
  "legacy": {
    "numericId": 1,
    "rawKeys": ["1", "001"]
  },
  "displayName": {
    "primary": "Character 001",
    "ja": null,
    "zhHans": null,
    "aliases": []
  },
  "profile": {
    "species": null,
    "title": null,
    "description": null
  },
  "dictionaryRefs": {
    "talents": [],
    "baseStats": [],
    "abilities": []
  },
  "resourceRefs": [],
  "dialogueSourceRefs": [],
  "sourceTrace": {},
  "review": {}
}
```

转换规则：

- 从资源数字前缀、角色 CSV、ERB 口上路径候选中收集 legacy numeric id。
- 如果多个来源给出同一 numeric id，合并为同一 character draft。
- 名称字段没有可靠来源时不猜测，使用 `Character 001` 类占位并要求复核。
- 资源绑定只建立候选，不假设图片类型一定正确。

## Location Schema

地点在 M2 只定义 schema，不强求从现有内容自动生成。

```json
{
  "id": "core.location.pending.human_village",
  "displayName": {
    "primary": "Pending Location",
    "zhHans": null,
    "ja": null
  },
  "kind": "unknown",
  "tags": [],
  "connections": [],
  "sourceTrace": {},
  "review": {
    "status": "needs_review"
  }
}
```

M2 允许地点全部进入人工草案。不要为了填满 schema 从 ERB 正文猜地点。

## Resource Schema

资源对象描述图片、音频、字体和说明文件，不保存二进制内容。

```json
{
  "id": "core.resource.001.face.default",
  "legacy": {
    "numericPrefix": 1,
    "variantTokens": ["face", "default"]
  },
  "mediaType": "image",
  "sourcePath": "resources/1_顔.webp",
  "usage": ["portrait"],
  "characterRefs": ["core.character.001"],
  "hash": null,
  "metadata": {
    "author": "unknown",
    "license": "unknown"
  },
  "sourceTrace": {},
  "review": {
    "status": "needs_review",
    "blockingIssues": ["license_unknown"]
  }
}
```

资源变体初步规则：

- 数字前缀候选绑定角色。
- `顔` 候选为 portrait/face。
- `立ち` 候选为 standing sprite。
- `巨`、服装、表情等后缀只作为 variant token，不直接映射玩法语义。
- 无数字前缀资源进入 `unbound_resource`。

## Dialogue Source Schema

M2 不转换 ERB 正文为 dialogue scene。先建立 dialogue source 索引，保留来源、角色候选、主题候选和转换难度。

```json
{
  "id": "core.dialogue_source.kojo.001.daily",
  "kind": "legacy_erb_source",
  "legacy": {
    "characterNumericId": 1,
    "filePattern": "M_KOJO_K1_*",
    "categoryTokens": ["daily"]
  },
  "sourcePath": "ERB/...",
  "candidateSpeakerRefs": ["core.character.001"],
  "candidateSceneKind": "daily",
  "conversionPlan": {
    "strategy": "manual_or_subset_erb",
    "requiresErbSubset": true,
    "requiresManualReview": true
  },
  "sourceTrace": {},
  "review": {}
}
```

DialogueScene schema 只为未来新内容和少量手工转换文本预留：

```json
{
  "id": "core.dialogue.scene.example",
  "entryNodeId": "node-001",
  "speakerRefs": [],
  "resourceRefs": [],
  "nodes": [
    {
      "id": "node-001",
      "speakerRef": null,
      "text": "",
      "choices": [],
      "effects": [],
      "conditions": []
    }
  ],
  "sourceTrace": {},
  "review": {}
}
```

M2 自动转换不能把 ERB 台词直接塞进 `text`。只有后续明确授权的转换器才能输出正文，并且输出位置必须是独立内容包仓库。

## Migration Report Schema

每次转换草案必须生成报告：

```json
{
  "schemaVersion": "migration-report/v1-draft",
  "sourceRootId": "eratw-content",
  "generatedAt": "2026-06-13T12:00:00+08:00",
  "summary": {
    "sourceFilesSeen": 6586,
    "objectsGenerated": 0,
    "objectsNeedingReview": 0,
    "unmappedItems": 0
  },
  "unmappedItems": [
    {
      "code": "CSV_SEMANTICS_UNKNOWN",
      "sourcePath": "CSV/Example.csv",
      "severity": "warning",
      "message": "CSV file classified but not mapped to a domain schema."
    }
  ]
}
```

原则：

- 未映射项必须显式报告。
- 低置信度转换不能伪装成 accepted。
- 报告不包含正文。

## 验证规则

M2 schema 验证分四层：

1. JSON Schema 结构校验。
2. ID 唯一性校验。
3. 引用完整性校验：character/resource/dialogue/dictionary refs 必须存在或标为 pending。
4. source trace 校验：自动生成对象必须能追溯到 source file 或 audit record。

阻断级错误：

- 重复 ID。
- 引用不存在且未标为 pending。
- 自动生成对象缺少 sourceTrace。
- 迁移过程丢弃字段但未写入 unmapped report。
- 输出路径落入 `ERAtw-NEXT` engine 仓库。

警告级错误：

- license/author unknown。
- 资源用途不确定。
- 名称占位。
- CSV 语义未知。
- ERB source 需要人工复核。

## 转换顺序

推荐第一批转换草案：

1. 生成 source file index。
2. 生成 dictionary draft。
3. 从资源数字前缀生成 character/resource 候选绑定。
4. 从 ERB 文件名生成 dialogue source 候选。
5. 生成 migration report。
6. 手工复核后再考虑生成 minimal content package。

不推荐在 M2 直接做：

- ERB 正文转换。
- 玩法字段推断。
- 关系/好感/事件效果推断。
- 地点自动推断。

## 与 M0/M1/M3 的关系

- M0 提供工程骨架，但 M2 文档不依赖 M0 代码。
- M1 提供只读事实输入，M2 使用其统计口径。
- M3 才实现最小内容包加载；M2 只定义并生成不可玩的包契约草案。

## 验收标准

- 有 M2 schema/转换草案 spec。
- 明确 package manifest、dictionary、character、location、resource、dialogue source、migration report 的边界。
- 明确自动转换、人工复核、不可自动迁移的判定。
- 明确不输出正文、不接 UI、不提交内容本体。
- 转换器拒绝将输出写入 engine 仓库。
- 转换器拒绝写入 M1 审计输入目录或覆盖已有输出目录；校验失败时不落盘。
- 真实数据生成结果通过 schema、唯一 ID、引用和 source trace 校验。
- 文档通过 `git diff --check`。

## 实施结果

- 输入：M1 的 6586 条 source file index 和资源审计报告。
- 输出对象：4194，其中角色 161、资源 1320、dialogue source 2685、CSV dictionary source 28。
- 未映射项：2553，全部显式写入 `unmapped-items.jsonl` 和 migration report。
- ID：4194 个全部唯一；同名 CSV、同内容资源和名称归一化碰撞均由相对路径 hash 消歧。
- 验证：0 error、2640 warning；1320 个资源的 author/license 未知分别生成 warning，必须人工治理。
- 地点与 dialogue scene 保持空草案，不从 ERB 正文猜测。
