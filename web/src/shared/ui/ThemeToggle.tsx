import { useCallback, useEffect, useState } from "react";
import type { ReactElement } from "react";
import themeLib from "@/shared/lib/theme";

type Theme = ReturnType<typeof themeLib.getStoredTheme>;

const order: Theme[] = ["system", "light", "dark"];

const labels: Record<Theme, string> = {
  dark: "Dark",
  light: "Light",
  system: "System",
};

const icons: Record<Theme, string> = {
  dark: "☾",
  light: "☀",
  system: "◐",
};

const nextTheme = (current: Theme): Theme => {
  const index = order.indexOf(current);
  const step = 1;
  return order[(index + step) % order.length] ?? "system";
};

const ThemeToggle = (): ReactElement => {
  const [theme, setTheme] = useState<Theme>("system");

  useEffect(() => {
    const initial = themeLib.getStoredTheme();
    setTheme(initial);
    themeLib.applyTheme(initial);
  }, []);

  const onClick = useCallback(() => {
    setTheme((current) => {
      const next = nextTheme(current);
      themeLib.setStoredTheme(next);
      return next;
    });
  }, []);

  return (
    <button
      aria-label={`Theme: ${labels[theme]}. Click to change.`}
      className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-border bg-surface text-text-muted transition hover:bg-surface-muted hover:text-text"
      onClick={onClick}
      title={`Theme: ${labels[theme]}`}
      type="button"
    >
      <span aria-hidden="true" className="text-base leading-none">
        {icons[theme]}
      </span>
    </button>
  );
};

export default ThemeToggle;
