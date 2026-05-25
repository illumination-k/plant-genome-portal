import type { ReactElement } from "react";
import datasetExport from "@/features/datasets/data/datasets";
import DatasetRow from "@/features/datasets/components/DatasetRow";

const DatasetTableBody = (): ReactElement => (
  <tbody className="divide-y divide-border-subtle">
    {datasetExport.datasets.map((dataset) => (
      <DatasetRow dataset={dataset} key={dataset.assembly} />
    ))}
  </tbody>
);

export default DatasetTableBody;
