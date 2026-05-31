import { geneOrthogroupsOptions } from "@/api/client/@tanstack/react-query.gen";
import type { GeneRecord, Orthogroup, OrthogroupMember } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import GeneIdLink from "@/shared/bio/GeneIdLink";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";
import geneFormat from "@/shared/lib/geneFormat";

const EMPTY = 0;

const isInternalMember = (member: OrthogroupMember, geneRecord: GeneRecord): boolean =>
  member.assembly_accession === geneRecord.gene.assembly_accession;

const renderGene = (member: OrthogroupMember, geneRecord: GeneRecord): ReactElement => {
  if (isInternalMember(member, geneRecord)) {
    return <GeneIdLink geneId={member.gene_id} />;
  }

  return <span className="font-mono text-[13px] font-medium text-text">{member.gene_id}</span>;
};

const renderOptionalValue = (value: string | null | undefined): ReactElement => {
  if (!value) {
    return <span className="text-text-subtle">-</span>;
  }

  return <span>{value}</span>;
};

const OrthogroupMemberRow = (props: {
  geneRecord: GeneRecord;
  member: OrthogroupMember;
}): ReactElement => (
  <tr className="border-t border-border-subtle hover:bg-surface-muted">
    <td className="px-4 py-3 text-[13px] text-text">{props.member.scientific_name}</td>
    <td className="px-4 py-3">{renderGene(props.member, props.geneRecord)}</td>
    <td className="px-4 py-3 text-[13px] text-text-muted">
      {renderOptionalValue(props.member.symbol)}
    </td>
    <td className="px-4 py-3 font-mono text-[12px] text-text-muted">
      {renderOptionalValue(props.member.assembly_accession)}
    </td>
  </tr>
);

const OrthogroupHeaderRow = (): ReactElement => (
  <tr>
    <th className="px-4 py-3 font-semibold">Species</th>
    <th className="px-4 py-3 font-semibold">Gene</th>
    <th className="px-4 py-3 font-semibold">Symbol</th>
    <th className="px-4 py-3 font-semibold">Assembly</th>
  </tr>
);

const OrthogroupTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-[12px] uppercase tracking-[0.08em] text-text-subtle">
    <OrthogroupHeaderRow />
  </thead>
);

const renderMember = (member: OrthogroupMember, geneRecord: GeneRecord): ReactElement => (
  <OrthogroupMemberRow
    geneRecord={geneRecord}
    key={`${member.tax_id}:${member.gene_id}`}
    member={member}
  />
);

const OrthogroupTableBody = (props: {
  geneRecord: GeneRecord;
  members: OrthogroupMember[];
}): ReactElement => (
  <tbody>{props.members.map((member) => renderMember(member, props.geneRecord))}</tbody>
);

const OrthogroupMemberTable = (props: {
  geneRecord: GeneRecord;
  members: OrthogroupMember[];
}): ReactElement => (
  <table className="w-full min-w-[680px] text-left text-sm">
    <OrthogroupTableHead />
    <OrthogroupTableBody geneRecord={props.geneRecord} members={props.members} />
  </table>
);

const OrthogroupTableShell = (props: {
  geneRecord: GeneRecord;
  group: Orthogroup;
}): ReactElement => (
  <div className="overflow-x-auto">
    <OrthogroupMemberTable geneRecord={props.geneRecord} members={props.group.members} />
  </div>
);

const OrthogroupCardHeader = (props: { group: Orthogroup }): ReactElement => (
  <header className="flex flex-wrap items-baseline gap-2">
    <h3 className="font-mono text-sm font-semibold text-text">{props.group.id}</h3>
    <span className="text-[12px] tabular-nums text-text-subtle">
      {props.group.members.length} members
    </span>
  </header>
);

const OrthogroupCard = (props: { geneRecord: GeneRecord; group: Orthogroup }): ReactElement => (
  <article className="flex flex-col gap-3 rounded-lg border border-border-subtle bg-surface p-5">
    <OrthogroupCardHeader group={props.group} />
    <OrthogroupTableShell geneRecord={props.geneRecord} group={props.group} />
  </article>
);

const renderLoading = (): ReactElement => (
  <section className="flex flex-col gap-3 rounded-lg border border-border-subtle bg-surface p-5">
    <Skeleton size="caption" />
    <Skeleton size="row" />
    <Skeleton size="row" />
  </section>
);

const renderEmpty = (): ReactElement => (
  <EmptyState
    description="No orthogroup membership is available for this gene in the current snapshot."
    title="No orthogroups"
  />
);

const renderGroups = (groups: Orthogroup[], geneRecord: GeneRecord): ReactElement => (
  <section className="flex flex-col gap-4">
    {groups.map((group) => (
      <OrthogroupCard geneRecord={geneRecord} group={group} key={group.id} />
    ))}
  </section>
);

const GeneOrthologsTab = (props: { geneRecord: GeneRecord }): ReactElement => {
  const query = useQuery(
    geneOrthogroupsOptions({ path: { gene_id: props.geneRecord.gene.id } }),
  );
  const groups = query.data ?? [];

  if (query.isLoading && groups.length === EMPTY) {
    return renderLoading();
  }

  if (query.error) {
    return (
      <ErrorState
        detail={geneFormat.getErrorMessage(query.error)}
        title="Orthogroups could not be loaded"
      />
    );
  }

  if (groups.length === EMPTY) {
    return renderEmpty();
  }

  return renderGroups(groups, props.geneRecord);
};

export default GeneOrthologsTab;
