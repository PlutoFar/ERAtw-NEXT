import { createTheme } from "@mui/material/styles";

// 暗色主题，贴合终端/游戏工具的观感。字符画地图使用等宽字体。
export const theme = createTheme({
  palette: {
    mode: "dark",
    background: { default: "#0d1117", paper: "#141b24" },
    primary: { main: "#6ec0ff" },
    secondary: { main: "#e0c341" },
    success: { main: "#6ec06e" },
    warning: { main: "#e08a3c" },
    error: { main: "#d65a5a" },
  },
  typography: {
    fontFamily: [
      '"Segoe UI"',
      "system-ui",
      '"Microsoft YaHei"',
      '"PingFang SC"',
      "sans-serif",
    ].join(","),
  },
  shape: { borderRadius: 8 },
});

// 字符画地图专用等宽字体栈（CJK 等宽尽量对齐）。
export const monospaceStack = [
  '"Cascadia Mono"',
  '"Consolas"',
  '"Sarasa Mono SC"',
  '"Noto Sans Mono CJK SC"',
  '"Microsoft YaHei Mono"',
  "monospace",
].join(",");
