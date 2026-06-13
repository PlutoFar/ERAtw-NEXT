# ERAtw-NEXT M1 只读内容审计设计

## 结论

M1 目标是建立安全、可复现的只读内容审计流程，为后续 schema 设计、资源 manifest、内容迁移和有限 ERB 子集策略提供事实输入。M1 不接 UI，不修改 M0 工程骨架，不执行 ERB，不转换内容，不复制 `eratw-content` 正文或资产。

输入源只允许 `D:\AICODE\eratw-content`。`D:\AICODE\eratw` 只作为可游玩对照版本，M1 默认不扫描。`D:\AICODE\ERAtw-modern` 和 `D:\AICODE\ERAtw-native-foundation` 仍为无关项目。

实现状态：已完成。CLI 已纳入 Rust workspace，并在真实源目录上扫描 6586 个文件。所有报告写入被 Git 忽略的生成目录，并在落盘前通过 JSON Schema 校验。

## 范围

M1 可做：

- 扫描文件元数据：路径、扩展名、大小、mtime、目录层级。
- 统计 CSV、ERB、ERH、resources、font、sound、README/文档类文件。
- 识别排除项：exe、dll、sav、log、cache、archive、runtime output。
- 识别路径风险：重解析点、路径穿越、过长路径、无扩展文件、非常规扩展。
- 采样编码状态和解码错误，但采样结果只进入数量、路径和错误类别，不进入正文。
- 统计 ERB 高频语法、函数、变量和资源引用，但输出只包含 token/category 级统计。
- 输出 JSON 与 Markdown 报告。

M1 不做：

- 不执行 ERB、Python、bat、exe、dll 或任何内容源脚本。
- 不访问网络。
- 不修改、格式化、移动、删除 `eratw-content` 文件。
- 不读取 `ERAtw-modern`、`ERAtw-native-foundation`。
- 不把 `eratw-content` 加为 submodule。
- 不生成可运行内容包。
- 不改 UI。

## 审计输入

默认输入：

```text
D:\AICODE\eratw-content
```

后续工具参数建议：

```text
--source D:\AICODE\eratw-content
--out reports\content-audit\<timestamp>
--profile m1-readonly
--no-network
--no-execute
```

`--source` 必须解析到 allowlist 内的真实目录。M1 工具应拒绝相对路径穿越、UNC 网络路径和不存在路径。符号链接、junction、reparse point 默认拒绝或跳过，并进入报告。

## 输出

建议输出目录：

```text
reports/content-audit/YYYY-MM-DD-HHMMSS/
  summary.json
  summary.md
  files.jsonl
  directories.json
  extensions.json
  risks.json
  erb-stats.json
  csv-stats.json
  resources.json
```

这些报告是生成产物，默认不入库。仓库只保留手工整理后的摘要文档，例如 `docs/reports/2026-06-13-eratw-content-inventory.md`。

## 数据模型草案

### FileRecord

```json
{
  "relativePath": "ERB/example.ERB",
  "kind": "erb",
  "extension": ".erb",
  "sizeBytes": 1234,
  "modifiedTime": "2026-06-09T02:42:17+08:00",
  "depth": 3,
  "flags": ["non_ascii_path"]
}
```

### RiskRecord

```json
{
  "code": "REPARSE_POINT_SKIPPED",
  "severity": "blocker",
  "relativePath": "resources/link",
  "message": "Reparse point skipped by readonly audit policy."
}
```

### Summary

```json
{
  "schemaVersion": "content-audit-summary/v1",
  "sourceRoot": "D:/AICODE/eratw-content",
  "generatedAt": "2026-06-13T12:00:00+08:00",
  "policy": {
    "readonly": true,
    "network": false,
    "execute": false
  },
  "totals": {
    "files": 0,
    "directories": 0,
    "bytes": 0
  },
  "extensions": [],
  "risks": []
}
```

## 分类规则

基础分类：

- `erb`: `.erb`
- `erb_header`: `.erh`
- `csv`: `.csv`
- `resource_image`: `.webp`, `.png`, `.jpg`, `.jpeg`
- `resource_audio`: `.mp3`, `.mid`, `.wav`, `.ogg`, `.flac`
- `font`: `.ttf`, `.ttc`, `.otf`
- `document`: `.txt`, `.md`, `.pdf`, `.docx`, `.xlsx`, `.xls`
- `config`: `.config`, `.cfg`, `.xml`, `.json`, `.toml`
- `tool_script`: `.py`, `.bat`, `.ps1`
- `runtime_binary`: `.exe`, `.dll`
- `runtime_state`: `.sav`, `.log`, `.dat`, cache-like names
- `archive`: `.zip`, `.7z`, `.rar`
- `unknown`: 其他扩展或无扩展

M1 可以报告 `tool_script`，但不能执行。`runtime_binary`、`runtime_state`、`archive` 默认进入排除项。

## ERB 审计策略

ERB 审计分三层：

1. 元数据层：文件数量、大小、目录分布、最大文件、无扩展邻接文件。
2. 轻量文本层：编码尝试、行数、空行、注释行、token/category 统计。
3. 语法候选层：函数定义、CALL、IF/ELSE、SELECTCASE、PRINT/PRINTFORM、变量引用、资源引用。

输出只保留 token、计数、位置摘要和错误类别，不输出原始台词或正文段落。

M1 不是 ERB 解释器。任何需要执行语义、变量求值、宏展开或运行时状态的分析都推迟到 M5。

## CSV 审计策略

CSV 审计目标是为 M2 schema 映射做准备：

- 文件名与类型初步分类。
- 行数、列数、空行、重复 key。
- 编码和分隔符风险。
- 特殊配置文件如 `_Rename.csv`、`_Replace.csv`、`VariableSize.csv` 单独标记。

M1 不转换 CSV，不改编码，不重写文件。

## 资源审计策略

资源审计目标：

- 统计图片、音频、字体数量和大小。
- 检查资源 ID 命名模式候选。
- 识别重复文件名、同名不同扩展、缺失引用候选。
- 记录 license/author 元数据缺口，但不猜测版权状态。

M1 可以计算 hash，但 hash 报告属于生成产物，不默认提交。

## 安全规则

- 所有扫描使用只读文件 API。
- 报告输出目录必须是内容源之外的新路径，拒绝覆盖已有目录。
- 默认不跟随 reparse point。
- 默认不访问网络。
- 默认不执行任何文件。
- 任何异常路径进入风险报告，不中断全局扫描，除非 source root 本身不安全。
- 报告不得包含内容正文。
- 报告不得包含秘密、用户本机 token、环境变量或绝对系统隐私路径；source root 例外，因为这是项目约定输入。

## 与 M0 的实现边界

M1 最初在独立 worktree/branch 中推进；M0 合并后，CLI 接入 root Rust workspace。实现不得修改：

- `apps/`
- `crates/`
- `schemas/system-status.schema.json`
- root `package.json`
- M0 UI 文件

允许修改 root `Cargo.toml`/`Cargo.lock` 以注册独立工具 crate，允许新增 M1 schema 与 fixture。

## 验收标准

- 有 M1 审计 spec。
- 有一次只读元数据盘点报告。
- 有一次真实全层审计：编码、ERB token、CSV 结构、资源 hash 和引用候选。
- 报告能说明统计口径，尤其是 `.git/` 排除规则。
- 报告不包含 ERB/CSV 正文。
- 生成报告通过 JSON Schema 自校验。
- 文档通过 `git diff --check`。
- 不修改 M0 UI 与 engine 行为。

## 实施结果

- 文件：6586；目录：1214；总大小：367049853 bytes。
- ERB：3752；ERH：79；文本行：4335400；解码失败：0。
- CSV：261；资源资产：1320；资源引用缺失候选：10。
- 风险：47，无 blocker；包括无扩展文件、长路径、工具脚本和缺失资源引用候选。
- 输出：`summary.json`、`summary.md`、`files.jsonl`、`directories.json`、`extensions.json`、`risks.json`、`erb-stats.json`、`csv-stats.json`、`resources.json`。
