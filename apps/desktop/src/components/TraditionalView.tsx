import { GameShell } from "./traditional/GameShell";
import type { TraditionalViewProps } from "./traditional/shellTypes";

export const TraditionalView = (props: TraditionalViewProps) => (
  <GameShell {...props} />
);
