import ExonHeaderRow from "@/components/ExonHeaderRow";
import type { ReactElement } from "react";

const ExonTableHead = (): ReactElement => (
  <thead className="bg-zinc-50 text-zinc-600">
    <ExonHeaderRow />
  </thead>
);

export default ExonTableHead;
