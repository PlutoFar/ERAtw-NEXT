import Box from "@mui/material/Box";
import Chip from "@mui/material/Chip";
import Divider from "@mui/material/Divider";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import { useMemo, useState } from "react";
import { useSettings, type MapRenderMode } from "../settings/SettingsContext";
import type { MapModel } from "../types";
import { AsciiMapRenderer } from "./AsciiMapRenderer";
import { legendByKey, nodeById, nodesInArea } from "./mapGeometry";
import { SvgMapRenderer } from "./SvgMapRenderer";

export function MapView({ model }: { model: MapModel }) {
  const { mapRenderMode, setMapRenderMode } = useSettings();
  const [activeAreaId, setActiveAreaId] = useState(model.defaultAreaId);
  const firstNodeOfDefault = nodesInArea(model, model.defaultAreaId)[0]?.id;
  const [selectedId, setSelectedId] = useState<string | undefined>(firstNodeOfDefault);
  const [hoveredId, setHoveredId] = useState<string | undefined>();

  const legend = useMemo(() => legendByKey(model), [model]);
  const selected = nodeById(model, selectedId);

  const changeArea = (areaId: string) => {
    setActiveAreaId(areaId);
    const inArea = nodesInArea(model, areaId);
    if (!inArea.some((n) => n.id === selectedId)) {
      setSelectedId(inArea[0]?.id);
    }
  };

  const rendererProps = {
    model,
    areaId: activeAreaId,
    selectedId,
    hoveredId,
    onSelect: setSelectedId,
    onHover: setHoveredId,
  };

  return (
    <Box sx={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <Stack
        direction="row"
        alignItems="center"
        spacing={2}
        sx={{ px: 2, py: 1, borderBottom: "1px solid rgba(255,255,255,0.08)" }}
      >
        <Tabs
          value={activeAreaId}
          onChange={(_, value) => changeArea(value)}
          variant="scrollable"
          scrollButtons="auto"
          sx={{ minHeight: 40 }}
        >
          {model.areas.map((area) => (
            <Tab key={area.id} value={area.id} label={area.label} sx={{ minHeight: 40, py: 0 }} />
          ))}
        </Tabs>
        <Box sx={{ flex: 1 }} />
        <Typography variant="caption" color="text.secondary">
          显示方式
        </Typography>
        <ToggleButtonGroup
          size="small"
          exclusive
          value={mapRenderMode}
          onChange={(_, value: MapRenderMode | null) => value && setMapRenderMode(value)}
          aria-label="地图显示方式"
        >
          <ToggleButton value="ascii" aria-label="字符画地图">
            字符画
          </ToggleButton>
          <ToggleButton value="svg" aria-label="SVG 地图">
            SVG
          </ToggleButton>
        </ToggleButtonGroup>
      </Stack>

      <Box sx={{ flex: 1, display: "grid", gridTemplateColumns: { xs: "1fr", md: "1fr 300px" }, minHeight: 0 }}>
        <Box sx={{ position: "relative", minHeight: 360 }}>
          {mapRenderMode === "ascii" ? (
            <AsciiMapRenderer {...rendererProps} />
          ) : (
            <SvgMapRenderer {...rendererProps} />
          )}
        </Box>

        <Paper
          square
          variant="outlined"
          sx={{ p: 2, overflow: "auto", borderTop: 0, borderRight: 0, borderBottom: 0 }}
          aria-label="地图侧栏"
        >
          {selected ? (
            <Stack spacing={1.5}>
              <Box>
                <Typography variant="overline" color="text.secondary">
                  选中地点
                </Typography>
                <Typography variant="h6">
                  {selected.glyph} {selected.label}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {selected.note}
                </Typography>
              </Box>
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                <Chip size="small" label={`地形：${selected.terrain}`} />
                <Chip size="small" label={`移动 ${selected.moveMinutes} 分`} />
                <Chip size="small" variant="outlined" label={`类型：${selected.kind}`} />
              </Stack>

              <Divider />
              <Box>
                <Typography variant="subtitle2" gutterBottom>
                  在场人物（{selected.occupants.length}）
                </Typography>
                {selected.occupants.length > 0 ? (
                  <Stack spacing={0.75}>
                    {selected.occupants.map((occ) => {
                      const entry = legend[occ.activity];
                      return (
                        <Stack key={occ.id} direction="row" spacing={1} alignItems="center">
                          <Box
                            sx={{
                              width: 10,
                              height: 10,
                              borderRadius: "50%",
                              bgcolor: entry?.color ?? "grey.500",
                            }}
                          />
                          <Typography variant="body2">{occ.label}</Typography>
                          <Typography variant="caption" color="text.secondary">
                            {entry?.label ?? occ.activity}
                          </Typography>
                        </Stack>
                      );
                    })}
                  </Stack>
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    无
                  </Typography>
                )}
              </Box>

              <Divider />
              <Box>
                <Typography variant="subtitle2" gutterBottom>
                  连接到
                </Typography>
                <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                  {selected.links.map((link) => {
                    const target = nodeById(model, link);
                    if (!target) {
                      return null;
                    }
                    return (
                      <Chip
                        key={link}
                        size="small"
                        clickable
                        label={target.label}
                        onClick={() => {
                          if (target.areaId !== activeAreaId) {
                            setActiveAreaId(target.areaId);
                          }
                          setSelectedId(target.id);
                        }}
                      />
                    );
                  })}
                </Stack>
              </Box>
            </Stack>
          ) : (
            <Typography variant="body2" color="text.secondary">
              点击地图上的地点查看详情。
            </Typography>
          )}

          <Divider sx={{ my: 2 }} />
          <Typography variant="subtitle2" gutterBottom>
            图例
          </Typography>
          <Stack spacing={0.75}>
            {model.legend.map((entry) => (
              <Stack key={entry.key} direction="row" spacing={1} alignItems="center">
                <Box
                  sx={{
                    width: 10,
                    height: 10,
                    borderRadius: "50%",
                    bgcolor: entry.color,
                  }}
                />
                <Typography variant="body2">{entry.label}</Typography>
                <Typography variant="caption" color="text.secondary">
                  {entry.glyph}
                </Typography>
              </Stack>
            ))}
          </Stack>
        </Paper>
      </Box>
    </Box>
  );
}
