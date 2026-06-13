import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Chip from "@mui/material/Chip";
import Divider from "@mui/material/Divider";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import type {
  Capability,
  CapabilityStatus,
  Milestone,
  MilestoneStatus,
  PathKind,
  PathPlaceholder,
  SystemStatus,
} from "../types";
import { monospaceStack } from "../theme";

const capabilityColor: Record<CapabilityStatus, "success" | "info" | "default"> = {
  available: "success",
  planned: "info",
  disabled: "default",
};
const capabilityText: Record<CapabilityStatus, string> = {
  available: "可用",
  planned: "计划中",
  disabled: "已禁用",
};
const milestoneColor: Record<MilestoneStatus, "success" | "warning" | "default"> = {
  done: "success",
  in_progress: "warning",
  planned: "default",
};
const milestoneText: Record<MilestoneStatus, string> = {
  done: "已完成",
  in_progress: "进行中",
  planned: "计划中",
};
const pathColor: Record<PathKind, "primary" | "info" | "default"> = {
  read_only: "primary",
  reference: "info",
  excluded: "default",
};
const pathText: Record<PathKind, string> = {
  read_only: "只读",
  reference: "参考",
  excluded: "排除",
};

const verifyCommands = [
  "cargo fmt --check",
  "cargo test --workspace",
  "npm test",
  "npm run typecheck",
  "npm run build",
];

function CapabilityRow({ capability }: { capability: Capability }) {
  return (
    <Stack direction="row" spacing={1.5} alignItems="flex-start">
      <Chip
        size="small"
        label={capabilityText[capability.status]}
        color={capabilityColor[capability.status]}
        variant={capability.status === "disabled" ? "outlined" : "filled"}
      />
      <Box>
        <Typography variant="body2" fontWeight={600}>
          {capability.label}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {capability.description}
        </Typography>
      </Box>
    </Stack>
  );
}

function MilestoneRow({ milestone }: { milestone: Milestone }) {
  return (
    <Stack direction="row" spacing={1.5} alignItems="flex-start">
      <Chip
        size="small"
        label={`${milestone.id} · ${milestoneText[milestone.status]}`}
        color={milestoneColor[milestone.status]}
        variant={milestone.status === "planned" ? "outlined" : "filled"}
      />
      <Box>
        <Typography variant="body2" fontWeight={600}>
          {milestone.title}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {milestone.summary}
        </Typography>
      </Box>
    </Stack>
  );
}

function PathRow({ path }: { path: PathPlaceholder }) {
  return (
    <Stack direction="row" spacing={1.5} alignItems="center">
      <Chip
        size="small"
        label={pathText[path.kind]}
        color={pathColor[path.kind]}
        variant={path.kind === "excluded" ? "outlined" : "filled"}
        sx={{ minWidth: 56 }}
      />
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="body2" fontWeight={600}>
          {path.label}
          <Typography component="span" variant="caption" sx={{ ml: 1, fontFamily: monospaceStack }} color="text.secondary">
            {path.value}
          </Typography>
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {path.note}
        </Typography>
      </Box>
    </Stack>
  );
}

export function SystemStatusScreen({ status }: { status: SystemStatus }) {
  return (
    <Box sx={{ maxWidth: 1100, mx: "auto", p: { xs: 2, md: 3 } }} aria-label="系统状态">
      <Paper
        elevation={0}
        sx={{
          p: 3,
          mb: 3,
          background: "linear-gradient(135deg, rgba(110,192,255,0.12), rgba(224,195,65,0.08))",
          border: "1px solid rgba(255,255,255,0.08)",
        }}
      >
        <Stack direction="row" spacing={2} alignItems="center" flexWrap="wrap">
          <Typography variant="h4" fontWeight={800} letterSpacing={1}>
            {status.app.name}
          </Typography>
          <Chip color="warning" label={`阶段 ${status.app.stage}`} />
          <Chip
            variant="outlined"
            label={`引擎 ${status.engine.name} ${status.engine.version}`}
            sx={{ fontFamily: monospaceStack }}
          />
          <Chip variant="outlined" label={`构建 ${status.build.profile}`} />
        </Stack>
        <Typography variant="body1" color="text.secondary" sx={{ mt: 1.5 }}>
          {status.app.tagline}
        </Typography>
      </Paper>

      <Box
        sx={{
          display: "grid",
          gap: 3,
          gridTemplateColumns: { xs: "1fr", md: "1fr 1fr" },
        }}
      >
        <Card variant="outlined">
          <CardContent>
            <Typography variant="overline" color="text.secondary">
              当前里程碑
            </Typography>
            <Typography variant="h6" gutterBottom>
              {status.currentMilestone} 路线
            </Typography>
            <Stack spacing={1.5} divider={<Divider flexItem />}>
              {status.milestones.map((milestone) => (
                <MilestoneRow key={milestone.id} milestone={milestone} />
              ))}
            </Stack>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Typography variant="overline" color="text.secondary">
              能力
            </Typography>
            <Typography variant="h6" gutterBottom>
              引擎能力
            </Typography>
            <Stack spacing={1.5} divider={<Divider flexItem />}>
              {status.capabilities.map((capability) => (
                <CapabilityRow key={capability.id} capability={capability} />
              ))}
            </Stack>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Typography variant="overline" color="text.secondary">
              路径占位
            </Typography>
            <Typography variant="h6" gutterBottom>
              内容边界
            </Typography>
            <Stack spacing={1.5} divider={<Divider flexItem />}>
              {status.paths.map((path) => (
                <PathRow key={path.id} path={path} />
              ))}
            </Stack>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Typography variant="overline" color="text.secondary">
              验证命令
            </Typography>
            <Typography variant="h6" gutterBottom>
              本地验证
            </Typography>
            <Box
              component="pre"
              sx={{
                m: 0,
                p: 1.5,
                borderRadius: 1,
                bgcolor: "rgba(0,0,0,0.35)",
                fontFamily: monospaceStack,
                fontSize: 13,
                lineHeight: 1.7,
                overflowX: "auto",
              }}
            >
              {verifyCommands.join("\n")}
            </Box>
            <Typography variant="caption" color="text.secondary" sx={{ mt: 1.5, display: "block" }}>
              eratw-content 为外部只读源，不复制进引擎仓库；modern / native 为无关项目，不作输入。
            </Typography>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
