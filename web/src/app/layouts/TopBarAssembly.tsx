import type { ReactElement } from "react";

const TopBarAssembly = (): ReactElement => (
  <button
    className="hidden rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-[12px] text-text-muted hover:border-border hover:text-text md:inline"
    title="Active assembly"
    type="button"
  >
    GCA_037833805.1 · MpTak1 v7.1
  </button>
);

export default TopBarAssembly;
