import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneIdLink from "@/shared/bio/GeneIdLink";

const subtitle = (gene: Gene): string => gene.symbol ?? gene.locus_tag ?? "";

const renderSubtitle = (text: string): ReactElement | false => {
  if (text) {
    return <p className="mt-1 text-xs text-text-muted">{text}</p>;
  }
  return false;
};

const GeneTitleCell = (props: { gene: Gene }): ReactElement => (
  <td className="px-4 py-3">
    <GeneIdLink geneId={props.gene.id} />
    {renderSubtitle(subtitle(props.gene))}
  </td>
);

export default GeneTitleCell;
