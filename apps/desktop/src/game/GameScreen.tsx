import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import LinearProgress from "@mui/material/LinearProgress";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Select from "@mui/material/Select";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import { Bed, Clock3, FolderOpen, Footprints, Play, Save } from "lucide-react";
import { useMemo, useState } from "react";
import type { ContentPackageIndex, GameCommand, GameState, SaveReport } from "../types";

interface GameScreenProps {
  content: ContentPackageIndex | null;
  state: GameState | null;
  onNewGame: () => Promise<void>;
  onCommand: (command: GameCommand) => Promise<void>;
  onSave: (path: string) => Promise<SaveReport>;
  onLoadSave: (path: string) => Promise<void>;
  onChooseSavePath: (defaultPath?: string) => Promise<string | null>;
  onChooseLoadPath: (defaultPath?: string) => Promise<string | null>;
}

function messageOf(error: unknown): string {
  return error && typeof error === "object" && "message" in error
    ? String((error as { message: unknown }).message)
    : String(error);
}

function formatClock(state: GameState): string {
  const hours = Math.floor(state.clock.minuteOfDay / 60).toString().padStart(2, "0");
  const minutes = (state.clock.minuteOfDay % 60).toString().padStart(2, "0");
  return `第 ${state.clock.day} 日 ${hours}:${minutes}`;
}

export function GameScreen({
  content,
  state,
  onNewGame,
  onCommand,
  onSave,
  onLoadSave,
  onChooseSavePath,
  onChooseLoadPath,
}: GameScreenProps) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [targetLocation, setTargetLocation] = useState("");
  const [savePath, setSavePath] = useState("");

  const currentLocation = content?.locations.find((item) => item.id === state?.currentLocationId);
  const moveTargets = useMemo(() => {
    if (!content || !currentLocation) return [];
    return currentLocation.connections
      .map((id) => content.locations.find((location) => location.id === id))
      .filter((item): item is NonNullable<typeof item> => Boolean(item));
  }, [content, currentLocation]);

  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await action();
    } catch (actionError) {
      setError(messageOf(actionError));
    } finally {
      setBusy(false);
    }
  };

  if (!content) {
    return <Box sx={{ maxWidth: 900, mx: "auto", p: 3 }}><Alert severity="info">先在“内容包”页加载一个包。</Alert></Box>;
  }

  if (!state) {
    return (
      <Box sx={{ maxWidth: 900, mx: "auto", p: 3 }}>
        {!content.playable ? <Alert severity="warning" sx={{ mb: 2 }}>当前包可浏览但不可启动。需要 accepted 复核、playable.core 能力和至少一个地点。</Alert> : null}
        {error ? <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert> : null}
        <Button
          variant="contained"
          startIcon={<Play size={18} />}
          disabled={busy || !content.playable}
          onClick={() => run(onNewGame)}
        >
          开始新游戏
        </Button>
      </Box>
    );
  }

  return (
    <Box sx={{ maxWidth: 1100, mx: "auto", p: { xs: 2, md: 3 } }}>
      {error ? <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert> : null}
      {notice ? <Alert severity="success" sx={{ mb: 2 }}>{notice}</Alert> : null}

      <Paper variant="outlined" sx={{ p: 2, mb: 2 }}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={2} alignItems={{ sm: "center" }}>
          <Box>
            <Typography variant="h5" fontWeight={700}>{currentLocation?.displayName ?? state.currentLocationId}</Typography>
            <Typography variant="body2" color="text.secondary">{formatClock(state)} · 回合 {state.turn}</Typography>
          </Box>
          <Box sx={{ flex: 1 }} />
          <Chip label={`金钱 ${state.player.money}`} />
          <Chip label={`待处理事件 ${state.eventQueue.length}`} />
        </Stack>
        <Stack direction="row" spacing={1.5} alignItems="center" sx={{ mt: 2 }}>
          <Typography variant="body2" sx={{ minWidth: 72 }}>体力 {state.player.energy}/{state.player.maxEnergy}</Typography>
          <LinearProgress variant="determinate" value={(state.player.energy / state.player.maxEnergy) * 100} sx={{ flex: 1, height: 8 }} />
        </Stack>
      </Paper>

      <Paper variant="outlined" sx={{ p: 2, mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={700} gutterBottom>命令</Typography>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5} alignItems={{ md: "center" }}>
          <Button startIcon={<Clock3 size={18} />} variant="outlined" disabled={busy} onClick={() => run(() => onCommand({ type: "wait", minutes: 30 }))}>等待 30 分钟</Button>
          <Button startIcon={<Bed size={18} />} variant="outlined" disabled={busy} onClick={() => run(() => onCommand({ type: "rest", minutes: 60 }))}>休息 60 分钟</Button>
          <FormControl size="small" sx={{ minWidth: 220 }}>
            <InputLabel id="move-target-label">移动目标</InputLabel>
            <Select labelId="move-target-label" label="移动目标" value={targetLocation} onChange={(event) => setTargetLocation(event.target.value)}>
              {moveTargets.map((location) => <MenuItem key={location.id} value={location.id}>{location.displayName}</MenuItem>)}
            </Select>
          </FormControl>
          <Button
            startIcon={<Footprints size={18} />}
            variant="contained"
            disabled={busy || targetLocation.length === 0}
            onClick={() => run(async () => {
              await onCommand({ type: "move", locationId: targetLocation, minutes: 10 });
              setTargetLocation("");
            })}
          >
            移动
          </Button>
        </Stack>
      </Paper>

      {state.recentEvents.length > 0 ? (
        <Alert severity="info" sx={{ mb: 2 }}>
          {state.recentEvents.map((event) => `${event.kind} (${event.id})`).join("；")}
        </Alert>
      ) : null}

      <Paper variant="outlined" sx={{ p: 2 }}>
        <Typography variant="subtitle1" fontWeight={700} gutterBottom>存档</Typography>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5}>
          <TextField
            fullWidth
            size="small"
            label="存档 JSON 绝对路径"
            value={savePath}
            onChange={(event) => setSavePath(event.target.value)}
            inputProps={{ "aria-label": "存档 JSON 绝对路径" }}
          />
          <Button
            startIcon={<Save size={18} />}
            variant="outlined"
            disabled={busy}
            onClick={async () => {
              const selected = await onChooseSavePath(savePath || undefined);
              if (selected) setSavePath(selected);
            }}
            sx={{ flexShrink: 0, whiteSpace: "nowrap" }}
          >
            选择保存位置
          </Button>
          <Button
            startIcon={<Save size={18} />}
            variant="contained"
            disabled={busy || savePath.trim().length === 0}
            onClick={() => run(async () => {
              const report = await onSave(savePath.trim());
              setNotice(`已写入 ${report.path}，回合 ${report.turn}`);
            })}
            sx={{ flexShrink: 0, whiteSpace: "nowrap" }}
          >
            保存
          </Button>
          <Button
            startIcon={<FolderOpen size={18} />}
            variant="outlined"
            disabled={busy}
            onClick={() => run(async () => {
              const selected = await onChooseLoadPath(savePath || undefined);
              if (!selected) return;
              setSavePath(selected);
              await onLoadSave(selected);
              setNotice("存档已加载并通过 replay 校验。");
            })}
            sx={{ flexShrink: 0, whiteSpace: "nowrap" }}
          >
            读取
          </Button>
        </Stack>
      </Paper>
    </Box>
  );
}
