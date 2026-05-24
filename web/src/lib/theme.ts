import { type InferOutput, picklist, safeParse } from "valibot";

const STORAGE_KEY = "pgp:theme";
const DEFAULT_CANVAS = "#faf9f6";

const themeSchema = picklist(["dark", "light", "system"]);
type Theme = InferOutput<typeof themeSchema>;

const readCanvas = (): string => {
  const value = globalThis
    .getComputedStyle(globalThis.document.documentElement)
    .getPropertyValue("--canvas")
    .trim();
  return value || DEFAULT_CANVAS;
};

const syncMetaThemeColor = (): void => {
  const meta = globalThis.document.querySelector('meta[name="theme-color"]');
  if (meta) {
    meta.setAttribute("content", readCanvas());
  }
};

const applyTheme = (theme: Theme): void => {
  const root = globalThis.document.documentElement;
  if (theme === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = theme;
  }
  syncMetaThemeColor();
};

const getStoredTheme = (): Theme => {
  const raw = globalThis.localStorage.getItem(STORAGE_KEY);
  const parsed = safeParse(themeSchema, raw);
  if (parsed.success) {
    return parsed.output;
  }
  return "system";
};

const setStoredTheme = (theme: Theme): void => {
  globalThis.localStorage.setItem(STORAGE_KEY, theme);
  applyTheme(theme);
};

const themeLib = {
  applyTheme,
  getStoredTheme,
  setStoredTheme,
};

export default themeLib;
