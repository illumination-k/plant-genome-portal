import type { ReactElement } from "react";

const LandingRecentList = (): ReactElement => (
  <div className="col-span-12 md:col-start-3 md:col-span-4">
    <h2 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
      Recent
    </h2>
    <p className="mt-3 rounded-md border border-border-subtle bg-surface px-3 py-3 text-sm text-text-muted">
      No recent searches yet.
    </p>
  </div>
);

export default LandingRecentList;
