import GeneHeaderRow from "@/components/GeneHeaderRow";
import type { ReactElement } from "react";

const GeneTableHead = (): ReactElement => (
  <thead className="bg-zinc-50 text-zinc-600">
    <GeneHeaderRow />
  </thead>
);

export default GeneTableHead;
