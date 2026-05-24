type Theme = "dark" | "light" | "system";

const STORAGE_KEY = "pgp:theme";

const isTheme = (value: string | null): value is Theme =>
  value === "light" || value === "dark" || value === "system";

const syncMetaThemeColor = (): void => {
  const meta = globalThis.document.querySelector('meta[name="theme-color"]');
  if (meta === null) {
    return;
  }
  const canvas = globalThis
    .getComputedStyle(globalThis.document.documentElement)
    .getPropertyValue("--canvas")
    .trim();
  if (canvas !== "") {
    meta.setAttribute("content", canvas);
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
  const value = globalThis.localStorage.getItem(STORAGE_KEY);
  if (isTheme(value)) {
    return value;
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
