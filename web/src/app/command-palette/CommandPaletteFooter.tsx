import type { ReactElement } from "react";

const CommandPaletteFooter = (): ReactElement => (
  <footer className="flex items-center justify-between gap-3 border-t border-border-subtle bg-canvas px-3 py-2 text-[11px] text-text-subtle">
    <span>
      <kbd className="font-mono">↵</kbd> open · <kbd className="font-mono">esc</kbd> close
    </span>
    <span>Plant Genome Portal</span>
  </footer>
);

export default CommandPaletteFooter;
