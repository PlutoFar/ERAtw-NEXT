import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import TablePagination from "@mui/material/TablePagination";
import Tabs from "@mui/material/Tabs";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import { FolderOpen } from "lucide-react";
import { useState } from "react";
import type { ContentPackageIndex } from "../types";

interface ContentPackageScreenProps {
  content: ContentPackageIndex | null;
  onLoad: (path: string) => Promise<void>;
  onChooseDirectory: () => Promise<string | null>;
}

type IndexTab = "characters" | "locations" | "resources";

function messageOf(error: unknown): string {
  return error && typeof error === "object" && "message" in error
    ? String((error as { message: unknown }).message)
    : String(error);
}

export function ContentPackageScreen({ content, onLoad, onChooseDirectory }: ContentPackageScreenProps) {
  const [path, setPath] = useState(content?.rootPath ?? "");
  const [tab, setTab] = useState<IndexTab>("characters");
  const [page, setPage] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setBusy(true);
    setError(null);
    try {
      await onLoad(path.trim());
    } catch (loadError) {
      setError(messageOf(loadError));
    } finally {
      setBusy(false);
    }
  };

  const rowsPerPage = 50;
  const activeCount = content
    ? tab === "characters"
      ? content.characters.length
      : tab === "locations"
        ? content.locations.length
        : content.resources.length
    : 0;
  const pageStart = page * rowsPerPage;

  return (
    <Box sx={{ maxWidth: 1280, mx: "auto", p: { xs: 2, md: 3 } }}>
      <Paper variant="outlined" sx={{ p: 2, mb: 2 }}>
        <Stack direction={{ xs: "column", md: "row" }} spacing={1.5} alignItems={{ md: "center" }}>
          <TextField
            fullWidth
            size="small"
            label="内容包目录"
            value={path}
            onChange={(event) => setPath(event.target.value)}
            inputProps={{ "aria-label": "内容包目录" }}
          />
          <Button
            variant="outlined"
            startIcon={<FolderOpen size={18} />}
            disabled={busy}
            onClick={async () => {
              const selected = await onChooseDirectory();
              if (selected) setPath(selected);
            }}
            sx={{ flexShrink: 0 }}
          >
            选择目录
          </Button>
          <Button
            variant="contained"
            disabled={busy || path.trim().length === 0}
            onClick={load}
            sx={{ flexShrink: 0 }}
          >
            {busy ? "加载中" : "加载内容包"}
          </Button>
        </Stack>
      </Paper>

      {error ? <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert> : null}

      {!content ? (
        <Alert severity="info">尚未加载内容包。</Alert>
      ) : (
        <>
          <Stack direction={{ xs: "column", md: "row" }} spacing={1.5} alignItems={{ md: "center" }} sx={{ mb: 2 }}>
            <Box sx={{ minWidth: 0 }}>
              <Typography variant="h5" fontWeight={700}>{content.package.displayName}</Typography>
              <Typography variant="body2" color="text.secondary">
                {content.package.packageId} · {content.package.version} · {content.rootPath}
              </Typography>
            </Box>
            <Box sx={{ flex: 1 }} />
            <Chip color={content.playable ? "success" : "warning"} label={content.playable ? "可启动" : "仅索引"} />
            <Chip variant="outlined" label={`复核 ${content.reviewStatus}`} />
          </Stack>

          {content.warnings.map((warning) => (
            <Alert key={warning} severity="warning" sx={{ mb: 1 }}>{warning}</Alert>
          ))}

          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{ my: 2 }}>
            <Chip label={`角色 ${content.counts.characters}`} />
            <Chip label={`地点 ${content.counts.locations}`} />
            <Chip label={`资源 ${content.counts.resources}`} />
            <Chip label={`对话源 ${content.counts.dialogueSources}`} />
            <Chip label={`对话场景 ${content.counts.dialogueScenes}`} />
            <Chip label={`字典 ${content.counts.dictionaries}`} />
          </Stack>

          <Tabs
            value={tab}
            onChange={(_, value: IndexTab) => {
              setTab(value);
              setPage(0);
            }}
            sx={{ borderBottom: 1, borderColor: "divider" }}
          >
            <Tab value="characters" label="角色" />
            <Tab value="locations" label="地点" />
            <Tab value="resources" label="资源" />
          </Tabs>

          <TableContainer component={Paper} variant="outlined" square sx={{ mt: 2 }}>
            <Table
              size="small"
              sx={{
                minWidth: tab === "characters" ? 760 : tab === "locations" ? 720 : 900,
                "& .MuiTableCell-root": { whiteSpace: "nowrap" },
              }}
            >
              {tab === "characters" ? (
                <>
                  <TableHead><TableRow><TableCell>名称</TableCell><TableCell>ID</TableCell><TableCell align="right">资源</TableCell><TableCell align="right">对话源</TableCell><TableCell>复核</TableCell></TableRow></TableHead>
                  <TableBody>
                    {content.characters.slice(pageStart, pageStart + rowsPerPage).map((item) => (
                      <TableRow key={item.id}><TableCell>{item.displayName}</TableCell><TableCell>{item.id}</TableCell><TableCell align="right">{item.resourceCount}</TableCell><TableCell align="right">{item.dialogueSourceCount}</TableCell><TableCell>{item.reviewStatus}</TableCell></TableRow>
                    ))}
                  </TableBody>
                </>
              ) : null}
              {tab === "locations" ? (
                <>
                  <TableHead><TableRow><TableCell>名称</TableCell><TableCell>ID</TableCell><TableCell>类型</TableCell><TableCell align="right">连接</TableCell><TableCell>复核</TableCell></TableRow></TableHead>
                  <TableBody>
                    {content.locations.slice(pageStart, pageStart + rowsPerPage).map((item) => (
                      <TableRow key={item.id}><TableCell>{item.displayName}</TableCell><TableCell>{item.id}</TableCell><TableCell>{item.kind}</TableCell><TableCell align="right">{item.connections.length}</TableCell><TableCell>{item.reviewStatus}</TableCell></TableRow>
                    ))}
                  </TableBody>
                </>
              ) : null}
              {tab === "resources" ? (
                <>
                  <TableHead><TableRow><TableCell>ID</TableCell><TableCell>类型</TableCell><TableCell>来源路径</TableCell><TableCell>作者</TableCell><TableCell>许可</TableCell></TableRow></TableHead>
                  <TableBody>
                    {content.resources.slice(pageStart, pageStart + rowsPerPage).map((item) => (
                      <TableRow key={item.id}><TableCell>{item.id}</TableCell><TableCell>{item.mediaType}</TableCell><TableCell>{item.sourcePath}</TableCell><TableCell>{item.author}</TableCell><TableCell>{item.license}</TableCell></TableRow>
                    ))}
                  </TableBody>
                </>
              ) : null}
            </Table>
            <TablePagination
              component="div"
              count={activeCount}
              page={Math.min(page, Math.max(0, Math.ceil(activeCount / rowsPerPage) - 1))}
              onPageChange={(_, nextPage) => setPage(nextPage)}
              rowsPerPage={rowsPerPage}
              rowsPerPageOptions={[rowsPerPage]}
              labelDisplayedRows={({ from, to, count }) => `${from}-${to} / ${count}`}
            />
          </TableContainer>
        </>
      )}
    </Box>
  );
}
