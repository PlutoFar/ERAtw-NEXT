import { Application, Container, Graphics, Text } from "pixi.js";
import { useEffect, useRef, useState } from "react";
import { displayText } from "../engine/displayText";
import type { Location, WorldState } from "../types";

interface ModernMapProps {
  world: WorldState;
}

const terrainColor: Record<string, number> = {
  street: 0x5a6472,
  interior: 0x875f3f,
  grass: 0x4f7f56,
};

const layoutLocation = (location: Location, index: number) => ({
  location,
  x: 72 + index * 174,
  y: index % 2 === 0 ? 94 : 210,
});

const destroyApp = (app: Application) => {
  try {
    app.destroy(true);
  } catch {
    // Pixi can throw while tearing down a partially initialized renderer in jsdom.
  }
};

export const ModernMap = ({ world }: ModernMapProps) => {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const [renderFailed, setRenderFailed] = useState(false);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }

    let destroyed = false;
    let app: Application | null = null;
    setRenderFailed(false);

    const render = async () => {
      const nextApp = new Application();
      try {
        await nextApp.init({
          background: "#172026",
          antialias: true,
          resizeTo: host,
        });
      } catch {
        destroyApp(nextApp);
        if (!destroyed) {
          setRenderFailed(true);
        }
        return;
      }

      if (destroyed) {
        destroyApp(nextApp);
        return;
      }

      app = nextApp;
      host.replaceChildren(nextApp.canvas);

      const stage = new Container();
      nextApp.stage.addChild(stage);

      const grid = new Graphics();
      grid.rect(30, 38, 570, 300).fill(0x22313a);
      grid.rect(30, 38, 570, 300).stroke({ color: 0x7fa0aa, width: 2 });
      stage.addChild(grid);

      world.locations.map(layoutLocation).forEach(({ location, x, y }) => {
        const node = new Graphics();
        node
          .roundRect(x, y, 132, 76, 8)
          .fill(terrainColor[location.terrain] ?? 0x53636f)
          .stroke({ color: 0xe4ecef, width: 2 });
        stage.addChild(node);

        const label = new Text({
          text: displayText(location.name),
          style: {
            fill: 0xffffff,
            fontFamily: "Microsoft YaHei, sans-serif",
            fontSize: 18,
            fontWeight: "600",
          },
        });
        label.x = x + 18;
        label.y = y + 22;
        stage.addChild(label);

        const characterHere = world.characters.some(
          (character) => character.location_id === location.id,
        );
        if (characterHere) {
          const marker = new Graphics();
          marker.circle(x + 104, y + 18, 13).fill(0xf6d365);
          marker.circle(x + 104, y + 18, 13).stroke({ color: 0x1b1b1b, width: 2 });
          stage.addChild(marker);
        }
      });

      const weatherLayer = new Graphics();
      if (world.clock.weather === "clear") {
        weatherLayer.circle(548, 75, 26).fill(0xffd166);
      } else if (world.clock.weather === "rain") {
        for (let i = 0; i < 18; i += 1) {
          weatherLayer
            .moveTo(410 + i * 10, 58 + (i % 4) * 18)
            .lineTo(402 + i * 10, 83 + (i % 4) * 18)
            .stroke({ color: 0x9dd9ff, width: 2 });
        }
      }
      stage.addChild(weatherLayer);
    };

    void render();

    return () => {
      destroyed = true;
      if (app) {
        destroyApp(app);
      }
    };
  }, [world]);

  return (
    <div className="modern-view">
      <div className="pixi-host" ref={hostRef} aria-label="modern map canvas">
        {renderFailed ? (
          <div className="modern-map-fallback">现代地图渲染不可用。</div>
        ) : null}
      </div>
    </div>
  );
};
