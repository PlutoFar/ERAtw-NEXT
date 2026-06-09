# M1 本地审计快照

本快照只记录规模和风险，不包含旧 ERAtw 正文、资产或派生内容。

## 输入

- Source: `D:\AICODE\ERAtw`
- Command: `python -m eratw_content_pipeline.cli audit-legacy --source D:\AICODE\ERAtw --out reports\legacy-audit --sample-text-bytes 4096 --max-issues 100`

## 规模

- Total files: 14282
- Total size: 626536495 bytes
- ERB: 3752
- CSV: 261
- Legacy headers: 80
- Images: 1311
- Audio: 4
- Fonts: 5
- Archives: 21
- Documents: 8
- Tool scripts: 3
- Legacy runtime or saves excluded: 13
- Referenced resource names sampled from text: 577
- Characters: 162
- Dialogue/reference ERB files: 2284
- Matched resource references: 486
- Missing resource references: 91

## 风险信号

- 非中文优先采样：98 条，主要集中在角色 CSV。
- 解码异常采样：1 条。
- 疑似乱码采样：1 条。
- 旧运行时、DLL、存档已识别为排除项。

## 下一步

- 扩展 `.erh`、资源 XML、角色 CSV 的专门解析。
- 将角色清单、口上清单、资源清单拆成单独报告。
- 为 `asset-manifest.draft.json` 增加许可和作者人工补录字段。
