import GeneHeaderRow from "@/features/genes/components/GeneHeaderRow";
import type { ReactElement } from "react";

const GeneTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-text-muted">
    <GeneHeaderRow />
  </thead>
);

export default GeneTableHead;
