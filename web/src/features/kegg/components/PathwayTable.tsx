import type { KeggPathwaySummary } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import EmptyState from "@/shared/ui/EmptyState";

const zero = 0;

const pathwayName = (pathway: KeggPathwaySummary): string =>
  pathway.pathway.name ?? "Unnamed pathway";

const PathwayTableHead = (): ReactElement => (
  <thead className="border-b border-border-subtle bg-surface-muted text-[11px] uppercase tracking-[0.08em] text-text-subtle">
    <tr>
      <th className="px-4 py-3 font-semibold">ID</th>
      <th className="px-4 py-3 font-semibold">Name</th>
      <th className="px-4 py-3 text-right font-semibold">KOs</th>
      <th className="px-4 py-3 text-right font-semibold">Genes</th>
    </tr>
  </thead>
);

const PathwayRow = (props: { pathway: KeggPathwaySummary }): ReactElement => (
  <tr className="hover:bg-surface-muted">
    <td className="px-4 py-3">
      <a
        className="font-mono text-text underline-offset-2 hover:underline"
        href={`/kegg/pathway/${props.pathway.pathway.id}`}
      >
        {props.pathway.pathway.id}
      </a>
    </td>
    <td className="px-4 py-3 text-text-muted">{pathwayName(props.pathway)}</td>
    <td className="px-4 py-3 text-right font-mono text-text-muted">{props.pathway.ko_count}</td>
    <td className="px-4 py-3 text-right font-mono text-text-muted">{props.pathway.gene_count}</td>
  </tr>
);

const PathwayTableBody = (props: { pathways: KeggPathwaySummary[] }): ReactElement => (
  <tbody className="divide-y divide-border-subtle">
    {props.pathways.map((pathway) => (
      <PathwayRow key={pathway.pathway.id} pathway={pathway} />
    ))}
  </tbody>
);

const PathwayTable = (props: { pathways: KeggPathwaySummary[] }): ReactElement => {
  if (props.pathways.length === zero) {
    return (
      <EmptyState description="No KEGG pathways match the current filter." title="No pathways" />
    );
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-border-subtle bg-surface">
      <table className="w-full border-collapse text-left text-sm">
        <PathwayTableHead />
        <PathwayTableBody pathways={props.pathways} />
      </table>
    </div>
  );
};

export default PathwayTable;
