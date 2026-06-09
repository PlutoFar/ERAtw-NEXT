# M1 内容审计计划

## 输入

旧 ERAtw 路径作为只读源，例如：

```powershell
python -m eratw_content_pipeline.cli audit-legacy --source D:\AICODE\ERAtw --out reports\legacy-audit
```

## 输出

- `legacy-audit-report.json`：完整机器可读报告。
- `legacy-file-inventory.csv`：文件清单，便于人工筛选。
- `asset-manifest.draft.json`：资产 manifest 草案。
- `character-inventory.json/csv`：从 `CSV/Chara`、`ERB/キャラデータ` 和个人口上目录归并出的角色清单。
- `dialogue-inventory.json/csv`：口上与事件文本 ERB 的结构清单，不包含旧正文。
- `resource-reference-report.json/csv`：文本中资源引用名与资产文件名的匹配/缺失报告。
- `dialogue-coverage-report.json/csv`：按角色聚合的口上覆盖、行数、语言风险和缺资源引用。
- `summary.md`：阶段摘要。

## 审计维度

- ERB：文件数、行数、函数标签、资源引用、编码、语言、疑似乱码。
- CSV：文件数、行数、编码、语言、疑似乱码。
- resources、sound、font：路径、大小、sha256、类型、资源标签启发。
- 角色：旧 ID、名称、呼名、角色 CSV、角色数据 ERB、个人口上文件路径。
- 口上：文件路径、类型、归属角色、行数、函数/标签数量、资源引用数量、语言和乱码风险。
- 口上覆盖：每个角色的个人口上文件数、总行数、风险文件数、缺失资源引用数。
- 资源引用：引用名、出现次数、匹配资产路径、缺失状态。
- sav、exe、dll：记录为排除项，不进入新运行时。

## 验收口径

M1 完成时，需要能对本地旧 ERAtw 源生成报告，并回答：

- 要复刻的 ERB/CSV 规模。
- 可复用资产规模和校验值。
- 哪些内容需要中文化、重写或人工许可确认。
- 哪些旧运行时/存档/二进制已被明确排除。
