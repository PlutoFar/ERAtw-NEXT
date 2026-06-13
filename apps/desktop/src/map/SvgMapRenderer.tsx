import Box from "@mui/material/Box";
import Chip from "@mui/material/Chip";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import { useMemo, useState } from "react";
import type { MapModel, NodeKind } from "../types";
import { dominantActivity, legendByKey, nodesInArea } from "./mapGeometry";

interface SvgMapRendererProps {
  model: MapModel;
  areaId: string;
  selectedId?: string;
  hoveredId?: string;
  onSelect: (id: string) => void;
  onHover: (id: string | undefined) => void;
}

const UNIT = 28;
const PAD = 1;
const MIN_SCALE = 0.6;
const MAX_SCALE = 2.2;

const kindColor: Record<NodeKind, string> = {
  home: "#c98b5e",
  shop: "#e0a13c",
  shrine: "#c45cc4",
  landmark: "#6ec0ff",
  gate: "#d65a5a",
  public: "#6ec06e",
  nature: "#6fae6f",
};

export function SvgMapRenderer({
  model,
  areaId,
  selectedId,
  hoveredId,
  onSelect,
  onHover,
}: SvgMapRendererProps) {
  const [scale, setScale] = useState(1);
  const areaNodes = useMemo(() => nodesInArea(model, areaId), [model, areaId]);
  const legend = useMemo(() => legendByKey(model), [model]);

  const geometry = useMemo(() => {
    if (areaNodes.length === 0) {
      return { width: UNIT, height: UNIT, pos: {} as Record<string, { cx: number; cy: number }> };
    }
    const minX = Math.min(...areaNodes.map((n) => n.x));
    const minY = Math.min(...areaNodes.map((n) => n.y));
    const maxX = Math.max(...areaNodes.map((n) => n.x));
    const maxY = Math.max(...areaNodes.map((n) => n.y));
    const cols = maxX - minX + 1 + PAD * 2;
    const rows = maxY - minY + 1 + PAD * 2;
    const pos: Record<string, { cx: number; cy: number }> = {};
    for (const node of areaNodes) {
      pos[node.id] = {
        cx: (node.x - minX + PAD) * UNIT + UNIT / 2,
        cy: (node.y - minY + PAD) * UNIT + UNIT / 2,
      };
    }
    return { width: cols * UNIT, height: rows * UNIT, pos };
  }, [areaNodes]);

  const inArea = useMemo(() => new Set(areaNodes.map((n) => n.id)), [areaNodes]);

  return (
    <Box sx={{ position: "relative", height: "100%", display: "flex", flexDirection: "column" }}>
      <Stack
        direction="row"
        spacing={0.5}
        sx={{ position: "absolute", top: 8, right: 8, zIndex: 3 }}
      >
        <Tooltip title="缩小">
          <span>
            <IconButton
              size="small"
              aria-label="缩小地图"
              onClick={() => setScale((s) => Math.max(MIN_SCALE, +(s - 0.2).toFixed(2)))}
              sx={{ bgcolor: "rgba(0,0,0,0.4)", fontSize: 18, fontWeight: 700, width: 30, height: 30 }}
            >
              −
            </IconButton>
          </span>
        </Tooltip>
        <Tooltip title="放大">
          <span>
            <IconButton
              size="small"
              aria-label="放大地图"
              onClick={() => setScale((s) => Math.min(MAX_SCALE, +(s + 0.2).toFixed(2)))}
              sx={{ bgcolor: "rgba(0,0,0,0.4)", fontSize: 18, fontWeight: 700, width: 30, height: 30 }}
            >
              +
            </IconButton>
          </span>
        </Tooltip>
      </Stack>

      <Box
        aria-label="SVG 地图"
        sx={{
          flex: 1,
          overflow: "auto",
          p: 2,
          bgcolor: "#0a0e14",
          backgroundImage: "radial-gradient(rgba(255,255,255,0.04) 1px, transparent 1px)",
          backgroundSize: "28px 28px",
        }}
      >
        <Box
          sx={{
            position: "relative",
            width: geometry.width,
            height: geometry.height,
            transform: `scale(${scale})`,
            transformOrigin: "0 0",
          }}
        >
          <svg
            width={geometry.width}
            height={geometry.height}
            viewBox={`0 0 ${geometry.width} ${geometry.height}`}
            role="img"
            aria-label="地图节点与连线"
          >
            {/* 连线 */}
            {areaNodes.map((node) =>
              node.links
                .filter((target) => inArea.has(target) && node.id < target)
                .map((target) => {
                  const a = geometry.pos[node.id];
                  const b = geometry.pos[target];
                  return (
                    <line
                      key={`${node.id}-${target}`}
                      x1={a.cx}
                      y1={a.cy}
                      x2={b.cx}
                      y2={b.cy}
                      stroke="rgba(150,170,190,0.45)"
                      strokeWidth={2}
                    />
                  );
                }),
            )}
            {/* 节点 */}
            {areaNodes.map((node) => {
              const p = geometry.pos[node.id];
              const isSelected = node.id === selectedId;
              const isHovered = node.id === hoveredId;
              const activity = dominantActivity(node);
              const dot = activity ? legend[activity]?.color : undefined;
              return (
                <g
                  key={node.id}
                  transform={`translate(${p.cx}, ${p.cy})`}
                  style={{ cursor: "pointer" }}
                  onClick={() => onSelect(node.id)}
                  onMouseEnter={() => onHover(node.id)}
                  onMouseLeave={() => onHover(undefined)}
                >
                  <circle
                    r={UNIT * 0.46}
                    fill={kindColor[node.kind]}
                    fillOpacity={isSelected ? 1 : 0.85}
                    stroke={isSelected ? "#ffffff" : isHovered ? "#6ec0ff" : "rgba(0,0,0,0.5)"}
                    strokeWidth={isSelected ? 3 : isHovered ? 2 : 1}
                  />
                  <text
                    textAnchor="middle"
                    dominantBaseline="central"
                    fontSize={UNIT * 0.5}
                    fill="#0a0e14"
                    fontWeight={700}
                  >
                    {node.glyph}
                  </text>
                  {dot ? <circle cx={UNIT * 0.34} cy={-UNIT * 0.34} r={4} fill={dot} stroke="#0a0e14" /> : null}
                </g>
              );
            })}
          </svg>

          {/* 浮动标签层（HTML / MUI Chip），与 SVG 同步缩放 */}
          {areaNodes.map((node) => {
            const p = geometry.pos[node.id];
            return (
              <Chip
                key={`chip-${node.id}`}
                label={node.label}
                size="small"
                onClick={() => onSelect(node.id)}
                onMouseEnter={() => onHover(node.id)}
                onMouseLeave={() => onHover(undefined)}
                color={node.id === selectedId ? "primary" : "default"}
                variant={node.id === selectedId ? "filled" : "outlined"}
                sx={{
                  position: "absolute",
                  left: p.cx,
                  top: p.cy + UNIT * 0.5,
                  transform: "translateX(-50%)",
                  cursor: "pointer",
                  bgcolor: node.id === selectedId ? undefined : "rgba(10,14,20,0.85)",
                  height: 20,
                  "& .MuiChip-label": { px: 1, fontSize: 11 },
                }}
              />
            );
          })}
        </Box>
      </Box>
    </Box>
  );
}
