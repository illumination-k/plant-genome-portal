import type { Gene } from "@/api/client/types.gen";
import GeneTableBody from "@/features/genes/components/GeneTableBody";
import GeneTableHead from "@/features/genes/components/GeneTableHead";
import type { ReactElement } from "react";

const GeneTable = (props: { genes: Gene[] }): ReactElement => (
  <div className="overflow-x-auto">
    <table className="w-full min-w-[760px] text-left text-sm">
      <GeneTableHead />
      <GeneTableBody genes={props.genes} />
    </table>
  </div>
);

export default GeneTable;
