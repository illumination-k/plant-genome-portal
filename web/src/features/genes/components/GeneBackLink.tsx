import type { ReactElement } from "react";

const GeneBackLink = (): ReactElement => (
  <a
    className="inline-flex items-center gap-1 text-[13px] text-text-muted hover:text-text"
    href="/genes"
  >
    ← Back to genes
  </a>
);

export default GeneBackLink;
