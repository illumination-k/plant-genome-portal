import type { ReactElement } from "react";

const GeneBackLink = (): ReactElement => (
  <a
    className="col-span-12 text-sm font-medium text-emerald-800 hover:text-emerald-950"
    href="/genes"
  >
    Back to genes
  </a>
);

export default GeneBackLink;
