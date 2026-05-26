import { geneKeggOptions } from "@/api/client/@tanstack/react-query.gen";
import type { GeneKeggOrthologyEntry } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import GeneKeggEntry from "@/features/genes/components/GeneKeggEntry";

const EMPTY = 0;

const renderEmpty = (): ReactElement => <span className="text-[12px] text-text-subtle">—</span>;

const renderLoading = (): ReactElement => (
  <span className="text-[12px] text-text-subtle">Loading…</span>
);

const renderEntries = (entries: GeneKeggOrthologyEntry[]): ReactElement => (
  <div className="flex flex-col gap-3">
    {entries.map((entry) => (
      <GeneKeggEntry entry={entry} key={entry.ko} />
    ))}
  </div>
);

const renderBody = (entries: GeneKeggOrthologyEntry[], isFetching: boolean): ReactElement => {
  if (isFetching && entries.length === EMPTY) {
    return renderLoading();
  }
  if (entries.length === EMPTY) {
    return renderEmpty();
  }
  return renderEntries(entries);
};

const GeneKeggCard = (props: { geneId: string }): ReactElement => {
  const { data, isFetching } = useQuery(geneKeggOptions({ path: { gene_id: props.geneId } }));
  const entries = data?.entries ?? [];

  return (
    <section className="flex flex-col gap-3 rounded-lg border border-border-subtle bg-surface p-5">
      <header className="flex items-baseline gap-2">
        <h4 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
          KEGG
        </h4>
        <span className="text-[12px] tabular-nums text-text-subtle">{entries.length}</span>
      </header>
      {renderBody(entries, isFetching)}
    </section>
  );
};

export default GeneKeggCard;
