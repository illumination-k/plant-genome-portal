import type { ReactElement } from "react";
import { useCallback } from "react";
import KbdKey from "@/ui/KbdKey";
import keyboardShortcuts from "@/lib/keyboardShortcuts";

const TopBarSearchTrigger = (): ReactElement => {
  const onClick = useCallback(() => {
    keyboardShortcuts.openPalette();
  }, []);

  return (
    <button
      aria-label="Open command palette"
      className="ml-auto hidden h-8 min-w-[280px] items-center justify-between gap-3 rounded-md border border-border bg-canvas px-3 text-left text-sm text-text-subtle transition hover:border-border-strong hover:text-text-muted md:flex"
      onClick={onClick}
      type="button"
    >
      <span>Search genes, accession, region…</span>
      <KbdKey>⌘K</KbdKey>
    </button>
  );
};

export default TopBarSearchTrigger;
