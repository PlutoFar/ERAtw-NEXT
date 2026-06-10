import { useMemo, useRef, useState } from "react";
import type { CSSProperties, MouseEvent, PointerEvent, WheelEvent } from "react";
import type { TextMapAction, TextMapArea } from "../../types";
import { buildAsciiMapModel } from "./viewModel";

interface AsciiMapViewportProps {
  area: TextMapArea | undefined;
  currentLocationId: string | undefined;
  hoveredLocationId: string | undefined;
  loading: boolean;
  onAction: (action: TextMapAction) => void;
  onContextMenu: (event: MouseEvent<HTMLButtonElement>, locationId: string) => void;
  onHoverLocation: (locationId: string | undefined) => void;
  onInspectLocation: (locationId: string) => void;
  onMoveLocation: (locationId: string) => void;
  onZoomChange: (zoom: number) => void;
  pinnedLocationId: string | undefined;
  selectedLocationId: string | undefined;
  zoom: number;
}

type MapStyle = CSSProperties & {
  "--ascii-columns": number;
  "--ascii-font-scale": number;
  "--ascii-rows": number;
};

type HotspotStyle = CSSProperties & {
  "--hotspot-column": number;
  "--hotspot-row": number;
  "--hotspot-width": number;
};

type CellStyle = CSSProperties & {
  gridColumn: number;
  gridRow: number;
};

type CanvasStyle = CSSProperties & {
  transform: string;
};

interface PanState {
  x: number;
  y: number;
}

interface DragState {
  moved: boolean;
  panX: number;
  panY: number;
  pointerId: number;
  startX: number;
  startY: number;
}

const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value));

const cellToneClass = (character: string) => {
  if (/[0-9]/.test(character)) {
    return "cell-location";
  }
  if ("全合".includes(character)) {
    return "cell-boundary";
  }
  if ("■".includes(character)) {
    return "cell-wall";
  }
  if ("□+＋=＝-┼：:三".includes(character)) {
    return "cell-road";
  }
  if ("┃│└┘┌┐─━═＝＼／|".includes(character)) {
    return "cell-detail";
  }
  if ("木森林".includes(character)) {
    return "cell-forest";
  }
  if ("~≈♨川".includes(character)) {
    return "cell-water";
  }
  if ("◇◆○●＠".includes(character)) {
    return "cell-marker";
  }
  if ("東东西西南北门門龍龙灯".includes(character)) {
    return "cell-waypoint";
  }
  if (/\p{Script=Han}/u.test(character)) {
    return "cell-building-label";
  }
  return "";
};

export const AsciiMapViewport = ({
  area,
  currentLocationId,
  hoveredLocationId,
  loading,
  onAction,
  onContextMenu,
  onHoverLocation,
  onInspectLocation,
  onMoveLocation,
  onZoomChange,
  pinnedLocationId,
  selectedLocationId,
  zoom,
}: AsciiMapViewportProps) => {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const suppressClickRef = useRef(false);
  const [pan, setPan] = useState<PanState>({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const model = useMemo(() => buildAsciiMapModel(area), [area]);
  const style: MapStyle = {
    "--ascii-columns": model.maxColumns,
    "--ascii-font-scale": 1,
    "--ascii-rows": model.rowCount,
  };
  const canvasStyle: CanvasStyle = {
    transform: `translate3d(${pan.x}px, ${pan.y}px, 0) scale(${zoom})`,
  };

  const clampPan = (nextPan: PanState, nextZoom = zoom): PanState => {
    const viewport = viewportRef.current;
    const canvas = canvasRef.current;
    if (!viewport || !canvas) {
      return nextPan;
    }

    const edgePadding = 32;
    const scaledWidth = canvas.offsetWidth * nextZoom;
    const scaledHeight = canvas.offsetHeight * nextZoom;
    const minX = Math.min(edgePadding, viewport.clientWidth - scaledWidth - edgePadding);
    const minY = Math.min(edgePadding, viewport.clientHeight - scaledHeight - edgePadding);

    return {
      x: clamp(nextPan.x, minX, edgePadding),
      y: clamp(nextPan.y, minY, edgePadding),
    };
  };

  const finishDrag = (event: PointerEvent<HTMLDivElement>) => {
    if (dragRef.current?.pointerId === event.pointerId) {
      if (!dragRef.current.moved) {
        suppressClickRef.current = false;
      }
      dragRef.current = null;
      setDragging(false);
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  return (
    <div
      className={["ascii-map-viewport", dragging ? "dragging" : ""]
        .filter(Boolean)
        .join(" ")}
      style={style}
      ref={viewportRef}
      lang="zh-Hans-CN"
      data-zoom={zoom.toFixed(2)}
      data-pan-x={Math.round(pan.x)}
      data-pan-y={Math.round(pan.y)}
      onPointerDown={(event) => {
        if (event.button !== 0) {
          return;
        }
        dragRef.current = {
          pointerId: event.pointerId,
          panX: pan.x,
          panY: pan.y,
          moved: false,
          startX: event.clientX,
          startY: event.clientY,
        };
        event.currentTarget.setPointerCapture(event.pointerId);
      }}
      onPointerMove={(event) => {
        const drag = dragRef.current;
        if (!drag || drag.pointerId !== event.pointerId) {
          return;
        }
        const deltaX = event.clientX - drag.startX;
        const deltaY = event.clientY - drag.startY;
        const moved = Math.abs(deltaX) > 2 || Math.abs(deltaY) > 2;
        if (moved) {
          drag.moved = true;
          suppressClickRef.current = true;
          setDragging(true);
        }
        setPan(clampPan({ x: drag.panX + deltaX, y: drag.panY + deltaY }));
      }}
      onPointerUp={finishDrag}
      onPointerCancel={finishDrag}
      onWheel={(event: WheelEvent<HTMLDivElement>) => {
        event.preventDefault();
        const direction = event.deltaY > 0 ? -1 : 1;
        const nextZoom = clamp(zoom + direction * 0.08, 0.72, 1.45);
        if (nextZoom === zoom) {
          return;
        }
        const rect = event.currentTarget.getBoundingClientRect();
        const cursorX = event.clientX - rect.left;
        const cursorY = event.clientY - rect.top;
        const scale = nextZoom / zoom;
        const nextPan = {
          x: cursorX - (cursorX - pan.x) * scale,
          y: cursorY - (cursorY - pan.y) * scale,
        };
        setPan(clampPan(nextPan, nextZoom));
        onZoomChange(nextZoom);
      }}
    >
      <pre className="ascii-map-source" aria-hidden="true">
        {model.lines.join("\n")}
      </pre>
      <div className="ascii-map-canvas" style={canvasStyle} ref={canvasRef}>
        <div
          className="ascii-map-grid"
          aria-label="era text map"
          lang="zh-Hans-CN"
          data-row-count={model.rowCount}
          data-column-count={model.maxColumns}
        >
          {model.gridRows.map((row, rowIndex) => (
            <div className="ascii-map-row" key={`${area?.id ?? "missing"}:${rowIndex}`}>
              {row.map((character, columnIndex) => (
                <span
                  aria-hidden="true"
                  className={
                    character === " " || character === "　"
                      ? "ascii-map-cell space"
                      : ["ascii-map-cell", cellToneClass(character)]
                          .filter(Boolean)
                          .join(" ")
                  }
                  style={
                    {
                      gridColumn: columnIndex + 1,
                      gridRow: rowIndex + 1,
                    } satisfies CellStyle
                  }
                  data-map-row={rowIndex}
                  data-map-column={columnIndex}
                  key={`${rowIndex}:${columnIndex}`}
                >
                  {character === " " || character === "　" ? "\u00a0" : character}
                </span>
              ))}
            </div>
          ))}
        </div>
        <div className="ascii-map-hotspots" aria-label="text map hotspots">
          {model.hotspots.map((hotspot) => {
            const locationId =
              hotspot.action.type === "move_to_location"
                ? hotspot.action.location_id
                : undefined;
            const isCurrent = locationId === currentLocationId;
            const isSelected = locationId === selectedLocationId;
            const isHovered = locationId === hoveredLocationId;
            const isPinned = locationId === pinnedLocationId;
            const hotspotStyle: HotspotStyle = {
              "--hotspot-column": hotspot.column,
              "--hotspot-row": hotspot.row,
              "--hotspot-width": hotspot.width,
            };

            return (
              <button
                key={hotspot.key}
                type="button"
                className={[
                  "ascii-map-hotspot",
                  isCurrent ? "current" : "",
                  isSelected ? "selected" : "",
                  isHovered ? "hovered" : "",
                  isPinned ? "pinned" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                style={hotspotStyle}
                onClick={(event) => {
                  event.stopPropagation();
                  if (suppressClickRef.current) {
                    suppressClickRef.current = false;
                    return;
                  }
                  if (locationId) {
                    onInspectLocation(locationId);
                  } else {
                    onAction(hotspot.action);
                  }
                }}
                onDoubleClick={() => {
                  if (locationId && !suppressClickRef.current) {
                    onMoveLocation(locationId);
                  }
                }}
                onContextMenu={(event) => {
                  if (locationId) {
                    onContextMenu(event, locationId);
                  }
                }}
                onFocus={() => {
                  if (locationId) {
                    onHoverLocation(locationId);
                  }
                }}
                onBlur={() => onHoverLocation(undefined)}
                onMouseEnter={() => {
                  if (locationId) {
                    onHoverLocation(locationId);
                  }
                }}
                onMouseLeave={() => onHoverLocation(undefined)}
                disabled={loading}
                aria-label={hotspot.label}
                data-location-id={hotspot.locationId ?? undefined}
                title={hotspot.label}
              >
                <span className="sr-only">{hotspot.label}</span>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
};
