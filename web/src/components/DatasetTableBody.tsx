import type { ReactElement } from "react";
import datasetExport from "@/data/datasets";
import DatasetRow from "@/components/DatasetRow";

const DatasetTableBody = (): ReactElement => (
  <tbody className="divide-y divide-zinc-200">
    {datasetExport.datasets.map((dataset) => (
      <DatasetRow dataset={dataset} key={dataset.assembly} />
    ))}
  </tbody>
);

export default DatasetTableBody;
