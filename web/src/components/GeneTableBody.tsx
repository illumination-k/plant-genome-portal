import type { Gene } from "@/api/client/types.gen";
import GeneRow from "@/components/GeneRow";
import type { ReactElement } from "react";

const GeneTableBody = (props: { genes: Gene[] }): ReactElement => (
  <tbody className="divide-y divide-zinc-200">
    {props.genes.map((gene) => (
      <GeneRow gene={gene} key={gene.id} />
    ))}
  </tbody>
);

export default GeneTableBody;
