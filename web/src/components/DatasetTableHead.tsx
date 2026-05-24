import DatasetHeaderRow from "@/components/DatasetHeaderRow";
import type { ReactElement } from "react";

const DatasetTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-text-muted">
    <DatasetHeaderRow />
  </thead>
);

export default DatasetTableHead;
