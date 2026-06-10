import { useEffect, useState } from "react";
import { GameScreen } from "./GameScreen";
import { PauseMenu } from "./PauseMenu";
import type { TraditionalViewProps } from "./shellTypes";
import { TitleScreen } from "./TitleScreen";

type ShellMode = "title" | "game";

export const GameShell = ({ services, world }: TraditionalViewProps) => {
  const [mode, setMode] = useState<ShellMode>("title");
  const [paused, setPaused] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || mode !== "game") {
        return;
      }
      event.preventDefault();
      setPaused((open) => !open);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [mode]);

  const enterGame = () => {
    setMode("game");
    setPaused(false);
  };

  const returnTitle = () => {
    setPaused(false);
    setMode("title");
  };

  return (
    <main className="app-shell">
      {mode === "title" ? (
        <TitleScreen onEnterGame={enterGame} services={services} />
      ) : (
        <>
          <GameScreen onPause={() => setPaused(true)} services={services} world={world} />
          <PauseMenu
            onResume={() => setPaused(false)}
            onReturnTitle={returnTitle}
            open={paused}
            services={services}
          />
        </>
      )}
    </main>
  );
};
