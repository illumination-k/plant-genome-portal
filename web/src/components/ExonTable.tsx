import type { Exon } from "@/api/client/types.gen";
import ExonTableBody from "@/components/ExonTableBody";
import ExonTableHead from "@/components/ExonTableHead";
import type { ReactElement } from "react";

const ExonTable = (props: { exons: Exon[] }): ReactElement => (
  <div className="overflow-x-auto">
    <table className="w-full min-w-[640px] text-left text-sm">
      <ExonTableHead />
      <ExonTableBody exons={props.exons} />
    </table>
  </div>
);

export default ExonTable;
