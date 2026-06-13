import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import { useMemo, useState } from "react";
import type { KeyboardEvent } from "react";
import { monospaceStack } from "../theme";
import type { MapModel } from "../types";
import {
  buildAsciiBuffer,
  dominantActivity,
  legendByKey,
  nearestNodeInDirection,
  nodesInArea,
  type Direction,
} from "./mapGeometry";

interface AsciiMapRendererProps {
  model: MapModel;
  areaId: string;
  selectedId?: string;
  hoveredId?: string;
  onSelect: (id: string) => void;
  onHover: (id: string | undefined) => void;
}

const MIN_CELL = 18;
const MAX_CELL = 40;
const DEFAULT_CELL = 26;

const arrowToDirection: Record<string, Direction> = {
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
};

export function AsciiMapRenderer({
  model,
  areaId,
  selectedId,
  hoveredId,
  onSelect,
  onHover,
}: AsciiMapRendererProps) {
  const [cell, setCell] = useState(DEFAULT_CELL);
  const buffer = useMemo(() => buildAsciiBuffer(model, areaId), [model, areaId]);
  const legend = useMemo(() => legendByKey(model), [model]);
  const areaNodes = useMemo(() => nodesInArea(model, areaId), [model, areaId]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const dir = arrowToDirection[event.key];
    if (!dir) {
      return;
    }
    event.preventDefault();
    const next = nearestNodeInDirection(areaNodes, selectedId, dir);
    if (next) {
      onSelect(next);
    }
  };

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
              onClick={() => setCell((c) => Math.max(MIN_CELL, c - 3))}
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
              onClick={() => setCell((c) => Math.min(MAX_CELL, c + 3))}
              sx={{ bgcolor: "rgba(0,0,0,0.4)", fontSize: 18, fontWeight: 700, width: 30, height: 30 }}
            >
              +
            </IconButton>
          </span>
        </Tooltip>
      </Stack>

      <Box
        role="grid"
        aria-label="字符画地图"
        tabIndex={0}
        onKeyDown={onKeyDown}
        sx={{
          flex: 1,
          overflow: "auto",
          p: 2,
          outline: "none",
          bgcolor: "#0a0e14",
          backgroundImage:
            "radial-gradient(rgba(255,255,255,0.04) 1px, transparent 1px)",
          backgroundSize: `${cell}px ${cell}px`,
        }}
      >
        <Box
          sx={{
            position: "relative",
            display: "grid",
            gridTemplateColumns: `repeat(${buffer.columns}, ${cell}px)`,
            gridAutoRows: `${cell}px`,
            width: "max-content",
            fontFamily: monospaceStack,
            userSelect: "none",
          }}
        >
          {buffer.cells.map((c) => {
            if (c.type === "empty") {
              return <span key={`${c.col}-${c.row}`} />;
            }
            if (c.type === "link") {
              return (
                <span
                  key={`${c.col}-${c.row}`}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    color: "rgba(150,170,190,0.55)",
                    fontSize: cell * 0.72,
                    lineHeight: 1,
                  }}
                >
                  {c.ch}
                </span>
              );
            }
            // node
            const node = model.nodes.find((n) => n.id === c.nodeId)!;
            const activity = dominantActivity(node);
            const dot = activity ? legend[activity]?.color : undefined;
            const isSelected = node.id === selectedId;
            const isHovered = node.id === hoveredId;
            return (
              <button
                key={`${c.col}-${c.row}`}
                type="button"
                aria-label={node.label}
                aria-pressed={isSelected}
                onClick={() => onSelect(node.id)}
                onMouseEnter={() => onHover(node.id)}
                onMouseLeave={() => onHover(undefined)}
                style={{
                  position: "relative",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  cursor: "pointer",
                  fontFamily: monospaceStack,
                  fontSize: cell * 0.78,
                  lineHeight: 1,
                  color: isSelected ? "#0a0e14" : "#e8eef5",
                  background: isSelected
                    ? "#6ec0ff"
                    : isHovered
                      ? "rgba(110,192,255,0.22)"
                      : "rgba(255,255,255,0.06)",
                  border: isSelected
                    ? "2px solid #6ec0ff"
                    : "1px solid rgba(255,255,255,0.18)",
                  borderRadius: 6,
                }}
              >
                {node.glyph}
                {dot ? (
                  <span
                    aria-hidden
                    style={{
                      position: "absolute",
                      right: 1,
                      top: 1,
                      width: Math.max(5, cell * 0.22),
                      height: Math.max(5, cell * 0.22),
                      borderRadius: "50%",
                      background: dot,
                      boxShadow: "0 0 0 1px rgba(0,0,0,0.5)",
                    }}
                  />
                ) : null}
              </button>
            );
          })}

          {/* 标签浮层：节点名显示在地图上，信息更丰富 */}
          {areaNodes.map((node) => {
            const pos = buffer.nodePositions[node.id];
            if (!pos) {
              return null;
            }
            return (
              <span
                key={`label-${node.id}`}
                aria-hidden
                style={{
                  position: "absolute",
                  left: pos.col * cell + cell / 2,
                  top: pos.row * cell + cell,
                  transform: "translateX(-50%)",
                  marginTop: 2,
                  pointerEvents: "none",
                  whiteSpace: "nowrap",
                  fontSize: Math.max(10, cell * 0.42),
                  color: node.id === selectedId ? "#6ec0ff" : "rgba(220,228,236,0.8)",
                  textShadow: "0 1px 2px #000, 0 0 2px #000",
                }}
              >
                {node.label}
              </span>
            );
          })}
        </Box>
      </Box>
    </Box>
  );
}
