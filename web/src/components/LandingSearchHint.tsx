import type { ReactElement } from "react";

const LandingSearchHint = (): ReactElement => (
  <p className="mt-3 text-center text-[13px] text-text-subtle">
    Tip: press
    <kbd className="mx-1 rounded border border-border-subtle bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] text-text-muted">
      /
    </kbd>
    anywhere to focus the search box.
  </p>
);

export default LandingSearchHint;
