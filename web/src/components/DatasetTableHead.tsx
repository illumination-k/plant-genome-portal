import DatasetHeaderRow from "@/components/DatasetHeaderRow";
import type { ReactElement } from "react";

const DatasetTableHead = (): ReactElement => (
  <thead className="bg-zinc-50 text-zinc-600">
    <DatasetHeaderRow />
  </thead>
);

export default DatasetTableHead;
