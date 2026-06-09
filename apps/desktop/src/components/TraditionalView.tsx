import type { WorldState } from "../types";

interface TraditionalViewProps {
  world: WorldState;
}

export const TraditionalView = ({ world }: TraditionalViewProps) => {
  const character = world.characters[0];

  return (
    <div className="traditional-view">
      <div className="ascii-map" aria-label="ASCII map">
        {world.locations.map((location) => {
          const isOccupied = location.id === character.location_id;
          return (
            <div className="ascii-cell" key={location.id}>
              <span>{isOccupied ? "人" : location.ascii_symbol}</span>
              <small>{location.name}</small>
            </div>
          );
        })}
      </div>

      <div className="log-panel">
        <h2>日志</h2>
        <ol>
          {world.event_log.slice(0, 8).map((entry, index) => (
            <li key={`${entry}-${index}`}>{entry}</li>
          ))}
        </ol>
      </div>
    </div>
  );
};
