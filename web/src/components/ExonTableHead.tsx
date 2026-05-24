import ExonHeaderRow from "@/components/ExonHeaderRow";
import type { ReactElement } from "react";

const ExonTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-text-muted">
    <ExonHeaderRow />
  </thead>
);

export default ExonTableHead;
