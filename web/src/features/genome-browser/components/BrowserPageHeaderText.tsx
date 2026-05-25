import type { ReactElement } from "react";

const BrowserPageHeaderText = (): ReactElement => (
  <div>
    <h1 className="text-[24px] font-bold leading-[32px] tracking-tight text-text">
      Genome browser
    </h1>
    <p className="mt-1 text-[13px] text-text-muted">
      JBrowse 2 against the active assembly. Pass{" "}
      <span className="font-mono">?loc=Chr1:1-100000</span> to deep-link a region.
    </p>
  </div>
);

export default BrowserPageHeaderText;
