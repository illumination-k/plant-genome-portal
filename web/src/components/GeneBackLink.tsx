import type { ReactElement } from "react";

const GeneBackLink = (): ReactElement => (
  <a
    className="col-span-12 text-sm font-medium text-primary-800 hover:text-primary-900"
    href="/genes"
  >
    Back to genes
  </a>
);

export default GeneBackLink;
