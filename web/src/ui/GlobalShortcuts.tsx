import type { ReactElement } from "react";
import { useCallback, useEffect, useState } from "react";
import CommandPalette from "@/ui/CommandPalette";
import keyboardShortcuts from "@/lib/keyboardShortcuts";

const GlobalShortcuts = (): ReactElement => {
  const [paletteOpen, setPaletteOpen] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      if (keyboardShortcuts.isPaletteShortcut(event)) {
        event.preventDefault();
        setPaletteOpen(true);
        return;
      }
      if (event.key === "/" && !keyboardShortcuts.isEditable(event.target)) {
        event.preventDefault();
        setPaletteOpen(true);
      }
    };
    const onOpenEvent = (): void => {
      setPaletteOpen(true);
    };
    globalThis.window.addEventListener("keydown", onKeyDown);
    globalThis.window.addEventListener(keyboardShortcuts.PALETTE_EVENT, onOpenEvent);
    return (): void => {
      globalThis.window.removeEventListener("keydown", onKeyDown);
      globalThis.window.removeEventListener(keyboardShortcuts.PALETTE_EVENT, onOpenEvent);
    };
  }, []);

  const onOpenChange = useCallback((open: boolean): void => {
    setPaletteOpen(open);
  }, []);

  return <CommandPalette onOpenChange={onOpenChange} open={paletteOpen} />;
};

export default GlobalShortcuts;
