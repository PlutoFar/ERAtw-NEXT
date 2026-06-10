import { useMemo, useRef, useState } from "react";
import type { CSSProperties, MouseEvent, PointerEvent, WheelEvent } from "react";
import type { TextMapAction, TextMapArea } from "../../types";
import {
  buildAsciiMapModel,
  terminalWidth,
  type AsciiMapHotspot,
  type AsciiMapLabel,
  type SemanticMapFeature,
} from "./viewModel";

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
  "--hotspot-height": number;
  "--hotspot-column": number;
  "--hotspot-row": number;
  "--hotspot-width": number;
};

type CellStyle = CSSProperties & {
  gridColumn: string;
  gridRow: number;
};

type LabelStyle = CSSProperties & {
  "--label-column": number;
  "--label-row": number;
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

const featureOrder: Record<SemanticMapFeature["kind"], number> = {
  trees: 0,
  river: 1,
  road: 2,
  plaza: 3,
  water: 4,
  building: 5,
  gate: 6,
  landmark: 7,
  boundary: 8,
};

const sortedFeatures = (features: SemanticMapFeature[]) =>
  [...features].sort((left, right) => featureOrder[left.kind] - featureOrder[right.kind]);

const featureCenter = (feature: SemanticMapFeature) => ({
  x: feature.column + feature.width / 2,
  y: feature.row + feature.height / 2,
});

const featureRect = (feature: SemanticMapFeature, inset = 0) => ({
  x: feature.column + inset,
  y: feature.row + inset,
  width: Math.max(0, feature.width - inset * 2),
  height: Math.max(0, feature.height - inset * 2),
});

const roofPoints = (feature: SemanticMapFeature) => {
  const x = feature.column;
  const y = feature.row;
  const width = feature.width;
  const roofHeight = Math.min(3.8, Math.max(2.4, feature.height * 0.32));
  return `${x - 0.6},${y + roofHeight} ${x + width / 2},${y} ${
    x + width + 0.6
  },${y + roofHeight}`;
};

const markerText = (label: AsciiMapLabel) => `${label.marker} ${label.text}`;

const labelWidth = (label: AsciiMapLabel, hotspot: AsciiMapHotspot | undefined) => {
  const textWidth = terminalWidth(markerText(label)) * 0.72 + 1.8;
  const minimum = hotspot ? Math.min(7, Math.max(4.6, hotspot.width - 1.2)) : 4.6;
  return Math.min(18, Math.max(minimum, textWidth));
};

const semanticLabelClass = ({
  hoveredLocationId,
  label,
  pinnedLocationId,
  selectedLocationId,
  currentLocationId,
}: {
  currentLocationId: string | undefined;
  hoveredLocationId: string | undefined;
  label: AsciiMapLabel;
  pinnedLocationId: string | undefined;
  selectedLocationId: string | undefined;
}) =>
  [
    "semantic-svg-label",
    label.locationId === currentLocationId ? "current" : "",
    label.locationId === selectedLocationId ? "selected" : "",
    label.locationId === hoveredLocationId ? "hovered" : "",
    label.locationId === pinnedLocationId ? "pinned" : "",
  ]
    .filter(Boolean)
    .join(" ");

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

const renderSemanticFeature = (feature: SemanticMapFeature) => {
  const rect = featureRect(feature);
  const inner = featureRect(feature, 0.7);
  const center = featureCenter(feature);

  if (feature.kind === "trees") {
    return (
      <rect
        className="semantic-svg-trees"
        key={feature.key}
        {...rect}
        data-feature-label={feature.label}
      />
    );
  }

  if (feature.kind === "boundary") {
    return (
      <rect
        className="semantic-svg-boundary"
        key={feature.key}
        {...rect}
        data-feature-label={feature.label}
      />
    );
  }

  if (feature.kind === "road") {
    const horizontal = feature.width >= feature.height;
    return (
      <g className="semantic-svg-road" data-feature-label={feature.label} key={feature.key}>
        <rect {...rect} rx="0.9" />
        <line
          x1={horizontal ? feature.column + 1 : center.x}
          x2={horizontal ? feature.column + feature.width - 1 : center.x}
          y1={horizontal ? center.y : feature.row + 1}
          y2={horizontal ? center.y : feature.row + feature.height - 1}
        />
      </g>
    );
  }

  if (feature.kind === "river") {
    const startX = feature.column;
    const endX = feature.column + feature.width;
    const y = feature.row + feature.height / 2;
    return (
      <g className="semantic-svg-river" data-feature-label={feature.label} key={feature.key}>
        <path
          d={`M ${startX} ${y} C ${startX + feature.width * 0.24} ${
            y - 2.1
          }, ${startX + feature.width * 0.45} ${y + 1.8}, ${
            startX + feature.width * 0.64
          } ${y - 0.6} S ${endX - 8} ${y + 1.8}, ${endX} ${y}`}
        />
        <path
          d={`M ${startX + 4} ${y + 1.1} C ${startX + feature.width * 0.35} ${
            y + 2.4
          }, ${startX + feature.width * 0.54} ${y - 1.2}, ${endX - 4} ${
            y + 1
          }`}
        />
      </g>
    );
  }

  if (feature.kind === "plaza") {
    return (
      <g className="semantic-svg-plaza" data-feature-label={feature.label} key={feature.key}>
        <rect {...rect} rx="1.2" />
        <circle cx={center.x} cy={center.y} r={Math.min(feature.width, feature.height) * 0.28} />
        <path d={`M ${feature.column + 2} ${center.y} H ${feature.column + feature.width - 2}`} />
        <path d={`M ${center.x} ${feature.row + 2} V ${feature.row + feature.height - 2}`} />
      </g>
    );
  }

  if (feature.kind === "water") {
    return (
      <g className="semantic-svg-water" data-feature-label={feature.label} key={feature.key}>
        <rect {...rect} rx="0.8" />
        <path d={`M ${feature.column + 1} ${center.y - 1.2} q 2 -1.2 4 0 t 4 0`} />
        <path d={`M ${feature.column + 1} ${center.y + 1.2} q 2 -1.2 4 0 t 4 0`} />
      </g>
    );
  }

  if (feature.kind === "gate") {
    return (
      <g className="semantic-svg-gate" data-feature-label={feature.label} key={feature.key}>
        <rect {...inner} rx="0.4" />
        <path d={`M ${feature.column + 1} ${feature.row + 1.2} H ${feature.column + feature.width - 1}`} />
        <path d={`M ${feature.column + 2.1} ${feature.row + 1.2} V ${feature.row + feature.height - 0.7}`} />
        <path d={`M ${feature.column + feature.width - 2.1} ${feature.row + 1.2} V ${feature.row + feature.height - 0.7}`} />
      </g>
    );
  }

  if (feature.kind === "landmark") {
    return (
      <g className="semantic-svg-landmark" data-feature-label={feature.label} key={feature.key}>
        <rect {...inner} rx="0.7" />
        <path d={roofPoints(feature)} />
        <circle cx={center.x} cy={center.y + 0.6} r={Math.min(feature.width, feature.height) * 0.22} />
      </g>
    );
  }

  return (
    <g className="semantic-svg-building" data-feature-label={feature.label} key={feature.key}>
      <rect x={feature.column + 0.5} y={feature.row + 2.3} width={feature.width - 1} height={feature.height - 2.8} rx="0.45" />
      <path d={roofPoints(feature)} />
      <rect x={feature.column + 1.2} y={feature.row + 3.2} width={feature.width - 2.4} height={feature.height - 4.4} rx="0.25" />
    </g>
  );
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
  const hotspotByLocationId = useMemo(
    () =>
      new Map(
        model.hotspots
          .filter((hotspot) => hotspot.locationId !== null)
          .map((hotspot) => [hotspot.locationId, hotspot] as const),
      ),
    [model.hotspots],
  );
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
      className={[
        "ascii-map-viewport",
        model.semanticLayout ? "semantic-map" : "",
        dragging ? "dragging" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      style={style}
      ref={viewportRef}
      lang="zh-Hans-CN"
      data-zoom={zoom.toFixed(2)}
      data-pan-x={Math.round(pan.x)}
      data-pan-y={Math.round(pan.y)}
      data-map-renderer={model.semanticLayout ? "semantic" : "text"}
      data-semantic-renderer={model.semanticLayout?.renderer}
      data-image-prompt={model.semanticLayout?.imagePrompt}
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
        {model.semanticLayout ? (
          <svg
            className="semantic-map-svg"
            role="img"
            aria-label="human village svg map"
            viewBox={`0 0 ${model.semanticLayout.columns} ${model.semanticLayout.rows}`}
          >
            <defs>
              <pattern id="semantic-tree-dots" width="3.2" height="3.2" patternUnits="userSpaceOnUse">
                <circle cx="0.7" cy="0.8" r="0.22" />
                <circle cx="2.3" cy="1.9" r="0.18" />
              </pattern>
              <pattern id="semantic-road-stripes" width="2.4" height="2.4" patternUnits="userSpaceOnUse" patternTransform="rotate(35)">
                <rect width="1.1" height="2.4" x="0" y="0" />
              </pattern>
              <filter id="semantic-soft-glow" x="-20%" y="-20%" width="140%" height="140%">
                <feDropShadow dx="0" dy="0" stdDeviation="0.6" floodColor="#5ad7ff" floodOpacity="0.28" />
              </filter>
            </defs>
            <rect className="semantic-svg-ground" x="0" y="0" width={model.semanticLayout.columns} height={model.semanticLayout.rows} />
            {sortedFeatures(model.semanticLayout.features).map(renderSemanticFeature)}
            <g className="semantic-svg-labels" aria-label="map labels">
              {model.labels.map((label) => {
                const hotspot = label.locationId
                  ? hotspotByLocationId.get(label.locationId)
                  : undefined;
                const center = {
                  x: hotspot ? hotspot.column + hotspot.width / 2 : label.column,
                  y: hotspot ? hotspot.row + hotspot.height / 2 : label.row,
                };
                const width = labelWidth(label, hotspot);
                const text = markerText(label);

                return (
                  <g
                    className={semanticLabelClass({
                      currentLocationId,
                      hoveredLocationId,
                      label,
                      pinnedLocationId,
                      selectedLocationId,
                    })}
                    data-location-id={label.locationId ?? undefined}
                    key={label.key}
                    transform={`translate(${center.x} ${center.y})`}
                  >
                    <rect x={-width / 2} y="-1.25" width={width} height="2.5" rx="0.45" />
                    <text textAnchor="middle" dominantBaseline="middle">
                      {text}
                    </text>
                  </g>
                );
              })}
            </g>
          </svg>
        ) : null}
        <div
          className="ascii-map-grid"
          aria-label="era text map"
          lang="zh-Hans-CN"
          data-row-count={model.rowCount}
          data-column-count={model.maxColumns}
        >
          {model.cells.map((cell) => (
            <span
              aria-hidden="true"
              className={
                cell.character === " " || cell.character === "　"
                  ? "ascii-map-cell space"
                  : ["ascii-map-cell", cellToneClass(cell.character)]
                      .filter(Boolean)
                      .join(" ")
              }
              style={
                {
                  gridColumn: `${cell.column + 1} / span ${cell.width}`,
                  gridRow: cell.row + 1,
                } satisfies CellStyle
              }
              data-map-row={cell.row}
              data-map-column={cell.column}
              data-map-width={cell.width}
              data-map-character={cell.character}
              key={cell.key}
            >
              {cell.character === " " || cell.character === "　" ? "\u00a0" : cell.character}
            </span>
          ))}
        </div>
        {!model.semanticLayout ? (
          <div className="ascii-map-labels" aria-label="map labels">
            {model.labels.map((label) => {
              const isCurrent = label.locationId === currentLocationId;
              const isSelected = label.locationId === selectedLocationId;
              const isHovered = label.locationId === hoveredLocationId;
              const isPinned = label.locationId === pinnedLocationId;
              const labelStyle: LabelStyle = {
                "--label-column": label.column,
                "--label-row": label.row,
              };

              return (
                <span
                  className={[
                    "ascii-map-label",
                    isCurrent ? "current" : "",
                    isSelected ? "selected" : "",
                    isHovered ? "hovered" : "",
                    isPinned ? "pinned" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  data-location-id={label.locationId ?? undefined}
                  key={label.key}
                  style={labelStyle}
                >
                  <span className="ascii-map-label-marker">{label.marker}</span>
                  <span className="ascii-map-label-text">{label.text}</span>
                </span>
              );
            })}
          </div>
        ) : null}
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
              "--hotspot-height": hotspot.height,
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
