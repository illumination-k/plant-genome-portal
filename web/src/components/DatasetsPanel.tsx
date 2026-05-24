import DatasetTable from "@/components/DatasetTable";
import type { ReactElement } from "react";

const DatasetsPanel = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6">
    <h2 className="text-2xl font-semibold">Datasets</h2>
    <DatasetTable />
  </div>
);

export default DatasetsPanel;
