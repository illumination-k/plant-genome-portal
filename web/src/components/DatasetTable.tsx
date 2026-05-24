import DatasetTableBody from "@/components/DatasetTableBody";
import DatasetTableHead from "@/components/DatasetTableHead";
import type { ReactElement } from "react";

const DatasetTable = (): ReactElement => (
  <div className="mt-5 overflow-hidden rounded-lg border border-border-subtle">
    <table className="w-full text-left text-sm">
      <DatasetTableHead />
      <DatasetTableBody />
    </table>
  </div>
);

export default DatasetTable;
