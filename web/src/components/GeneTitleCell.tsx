import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const getGeneLabel = (gene: Gene): string => gene.symbol ?? gene.locus_tag ?? gene.id;

const GeneTitleCell = (props: { gene: Gene }): ReactElement => (
  <td className="px-4 py-3">
    <a
      className="font-semibold text-emerald-800 hover:text-emerald-950"
      href={`/genes/${props.gene.id}`}
    >
      {getGeneLabel(props.gene)}
    </a>
    <p className="mt-1 text-xs text-zinc-500">{props.gene.id}</p>
  </td>
);

export default GeneTitleCell;
