import { useMemo } from "react";
import type { CSSProperties, MouseEvent } from "react";
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
  pinnedLocationId,
  selectedLocationId,
  zoom,
}: AsciiMapViewportProps) => {
  const model = useMemo(() => buildAsciiMapModel(area), [area]);
  const style: MapStyle = {
    "--ascii-columns": model.maxColumns,
    "--ascii-font-scale": zoom,
    "--ascii-rows": model.rowCount,
  };

  return (
    <div className="ascii-map-viewport" style={style}>
      <pre className="ascii-map-source" aria-hidden="true">
        {model.lines.join("\n")}
      </pre>
      <div
        className="ascii-map-grid"
        aria-label="era text map"
        data-row-count={model.rowCount}
        data-column-count={model.maxColumns}
      >
        {model.gridRows.map((row, rowIndex) => (
          <div className="ascii-map-row" key={`${area?.id ?? "missing"}:${rowIndex}`}>
            {row.map((character, columnIndex) => (
              <span
                className={
                  character === " " || character === "　"
                    ? "ascii-map-cell space"
                    : "ascii-map-cell"
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
                if (locationId) {
                  onInspectLocation(locationId);
                } else {
                  onAction(hotspot.action);
                }
              }}
              onDoubleClick={() => {
                if (locationId) {
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
  );
};
