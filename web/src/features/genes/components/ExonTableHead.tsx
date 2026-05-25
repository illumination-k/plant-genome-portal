import ExonHeaderRow from "@/features/genes/components/ExonHeaderRow";
import type { ReactElement } from "react";

const ExonTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-text-muted">
    <ExonHeaderRow />
  </thead>
);

export default ExonTableHead;
