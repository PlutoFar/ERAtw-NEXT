import Alert from "@mui/material/Alert";
import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import Drawer from "@mui/material/Drawer";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { Settings } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { ContentPackageScreen } from "./content/ContentPackageScreen";
import { defaultEngineClient, type EngineClient } from "./engine/client";
import { GameScreen } from "./game/GameScreen";
import { MapView } from "./map/MapView";
import { useSettings, type MapRenderMode } from "./settings/SettingsContext";
import { SystemStatusScreen } from "./system/SystemStatusScreen";
import type {
  ContentPackageIndex,
  GameCommand,
  GameState,
  MapModel,
  SaveReport,
  SystemStatus,
} from "./types";

type LoadState =
  | { phase: "loading" }
  | { phase: "error"; message: string }
  | { phase: "ready"; status: SystemStatus; map: MapModel };

interface AppProps {
  client?: EngineClient;
}

function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}

export default function App({ client = defaultEngineClient }: AppProps) {
  const [state, setState] = useState<LoadState>({ phase: "loading" });
  const [tab, setTab] = useState<"status" | "map" | "content" | "game">("status");
  const [content, setContent] = useState<ContentPackageIndex | null>(null);
  const [game, setGame] = useState<GameState | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const { mapRenderMode, setMapRenderMode } = useSettings();

  const load = useCallback(() => {
    let cancelled = false;
    setState({ phase: "loading" });
    Promise.all([
      client.getSystemStatus(),
      client.getMapOverview(),
      client.getLoadedContent(),
      client.getGameState(),
    ])
      .then(([status, map, loadedContent, gameState]) => {
        if (!cancelled) {
          setState({ phase: "ready", status, map });
          setContent(loadedContent);
          setGame(gameState);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setState({ phase: "error", message: errorMessage(err) });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [client]);

  useEffect(() => load(), [load]);

  const loadContent = async (path: string) => {
    const loaded = await client.loadContentPackage(path);
    setContent(loaded);
    setGame(null);
  };

  const newGame = async () => {
    setGame(await client.newGame());
  };

  const applyCommand = async (command: GameCommand) => {
    const result = await client.applyGameCommand(command);
    setGame(result.state);
  };

  const writeSave = (path: string): Promise<SaveReport> => client.writeSave(path);

  const loadSave = async (path: string) => {
    setGame(await client.loadSave(path));
  };

  return (
    <Box sx={{ height: "100vh", display: "flex", flexDirection: "column", bgcolor: "background.default" }}>
      <AppBar position="static" color="transparent" elevation={0} sx={{ borderBottom: "1px solid rgba(255,255,255,0.08)" }}>
        <Toolbar variant="dense" sx={{ gap: 1.5 }}>
          <Typography variant="h6" fontWeight={800} letterSpacing={1} noWrap>
            ERAtw-NEXT
          </Typography>
          <Chip size="small" color="warning" label="M4" />
          <Box sx={{ flex: 1 }} />
          {state.phase === "ready" ? (
            <Tabs
              value={tab}
              onChange={(_, v) => setTab(v)}
              variant="scrollable"
              scrollButtons={false}
              sx={{ minHeight: 48, display: { xs: "none", md: "flex" } }}
            >
              <Tab value="status" label="状态" sx={{ minHeight: 48 }} />
              <Tab value="map" label="地图" sx={{ minHeight: 48 }} />
              <Tab value="content" label="内容包" sx={{ minHeight: 48 }} />
              <Tab value="game" label="游戏" sx={{ minHeight: 48 }} />
            </Tabs>
          ) : null}
          <IconButton aria-label="设置" onClick={() => setSettingsOpen(true)} sx={{ fontSize: 20 }}>
            <Settings size={20} />
          </IconButton>
        </Toolbar>
        {state.phase === "ready" ? (
          <Tabs
            value={tab}
            onChange={(_, v) => setTab(v)}
            variant="fullWidth"
            sx={{ minHeight: 44, display: { xs: "flex", md: "none" } }}
          >
            <Tab value="status" label="状态" sx={{ minHeight: 44, minWidth: 0 }} />
            <Tab value="map" label="地图" sx={{ minHeight: 44, minWidth: 0 }} />
            <Tab value="content" label="内容包" sx={{ minHeight: 44, minWidth: 0 }} />
            <Tab value="game" label="游戏" sx={{ minHeight: 44, minWidth: 0 }} />
          </Tabs>
        ) : null}
      </AppBar>

      <Box sx={{ flex: 1, minHeight: 0, overflow: "auto" }}>
        {state.phase === "loading" ? (
          <Stack alignItems="center" justifyContent="center" sx={{ height: "100%" }} spacing={2}>
            <CircularProgress />
            <Typography color="text.secondary">正在读取引擎状态…</Typography>
          </Stack>
        ) : null}

        {state.phase === "error" ? (
          <Box sx={{ maxWidth: 640, mx: "auto", mt: 6, px: 2 }}>
            <Alert
              severity="error"
              action={
                <Button color="inherit" size="small" onClick={() => load()}>
                  重试
                </Button>
              }
            >
              引擎状态不可用：{state.message}
            </Alert>
          </Box>
        ) : null}

        {state.phase === "ready" ? (
          tab === "status" ? (
            <SystemStatusScreen status={state.status} />
          ) : tab === "map" ? (
            <Box sx={{ height: "100%" }}>
              <MapView model={state.map} />
            </Box>
          ) : tab === "content" ? (
            <ContentPackageScreen
              content={content}
              onLoad={loadContent}
              onChooseDirectory={client.chooseContentPackageDirectory}
            />
          ) : (
            <GameScreen
              content={content}
              state={game}
              onNewGame={newGame}
              onCommand={applyCommand}
              onSave={writeSave}
              onLoadSave={loadSave}
              onChooseSavePath={client.chooseSavePath}
              onChooseLoadPath={client.chooseLoadSavePath}
            />
          )
        ) : null}
      </Box>

      <Drawer anchor="right" open={settingsOpen} onClose={() => setSettingsOpen(false)}>
        <Box sx={{ width: 320, p: 3 }} role="presentation">
          <Typography variant="h6" gutterBottom>
            设置
          </Typography>
          <Typography variant="subtitle2" sx={{ mt: 2 }} gutterBottom>
            地图显示方式
          </Typography>
          <ToggleButtonGroup
            fullWidth
            size="small"
            exclusive
            value={mapRenderMode}
            onChange={(_, value: MapRenderMode | null) => value && setMapRenderMode(value)}
            aria-label="地图显示方式设置"
          >
            <ToggleButton value="ascii">字符画</ToggleButton>
            <ToggleButton value="svg">SVG</ToggleButton>
          </ToggleButtonGroup>
          <Typography variant="caption" color="text.secondary" sx={{ mt: 2, display: "block" }}>
            可随时切换；两种模式共享同一份地图数据与选中状态。
          </Typography>
        </Box>
      </Drawer>
    </Box>
  );
}
