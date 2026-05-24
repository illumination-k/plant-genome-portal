import type { ReactElement } from "react";

const TopBarSearchTrigger = (): ReactElement => (
  <button
    aria-label="Open command palette"
    className="ml-auto hidden h-8 min-w-[280px] items-center justify-between gap-3 rounded-md border border-border bg-canvas px-3 text-left text-sm text-text-subtle transition hover:border-border-strong hover:text-text-muted md:flex"
    type="button"
  >
    <span>Search genes, accession, region…</span>
    <kbd className="rounded border border-border-subtle bg-surface px-1.5 py-0.5 font-mono text-[11px] text-text-muted">
      ⌘K
    </kbd>
  </button>
);

export default TopBarSearchTrigger;
