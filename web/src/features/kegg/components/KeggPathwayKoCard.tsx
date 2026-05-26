import type { KeggGeneSummary, KeggPathwayKoEntry } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import KeggPathwayGeneRow from "@/features/kegg/components/KeggPathwayGeneRow";

const SINGLE = 1;
const EMPTY = 0;

const pluralize = (count: number, singular: string, plural: string): string => {
  if (count === SINGLE) {
    return singular;
  }
  return plural;
};

const renderEmpty = (): ReactElement => (
  <p className="text-[12px] text-text-subtle">No genes in this dataset.</p>
);

const renderGeneList = (genes: KeggGeneSummary[]): ReactElement => (
  <ul className="flex flex-col">
    {genes.map((gene) => (
      <KeggPathwayGeneRow gene={gene} key={gene.id} />
    ))}
  </ul>
);

const renderBody = (genes: KeggGeneSummary[]): ReactElement => {
  if (genes.length === EMPTY) {
    return renderEmpty();
  }
  return renderGeneList(genes);
};

const KeggPathwayKoCard = (props: { entry: KeggPathwayKoEntry }): ReactElement => (
  <article className="flex flex-col gap-2 rounded-lg border border-border-subtle bg-surface p-4">
    <header className="flex items-baseline gap-2">
      <a
        className="font-mono text-[13px] text-text underline-offset-2 hover:underline"
        href={`https://www.kegg.jp/entry/${props.entry.ko}`}
        rel="noreferrer"
        target="_blank"
      >
        {props.entry.ko}
      </a>
      <span className="text-[12px] tabular-nums text-text-subtle">
        {props.entry.genes.length} {pluralize(props.entry.genes.length, "gene", "genes")}
      </span>
    </header>
    {renderBody(props.entry.genes)}
  </article>
);

export default KeggPathwayKoCard;
