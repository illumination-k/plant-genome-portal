import { keggPathwayOptions } from "@/api/client/@tanstack/react-query.gen";
import type { KeggPathwayDetail } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import { minLength, pipe, string } from "valibot";
import KeggPathwayBody from "@/features/kegg/components/KeggPathwayBody";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";
import useValidatedParam from "@/shared/lib/useValidatedParam";

const MIN_PATHWAY_ID_LENGTH = 1;
const ZERO = 0;
const SINGLE = 1;
const pathwayIdSchema = pipe(string(), minLength(MIN_PATHWAY_ID_LENGTH));

const pluralize = (count: number, singular: string, plural: string): string =>
  count === SINGLE ? singular : plural;

const renderPathwayName = (name: string | null | undefined): ReactElement | false => {
  if (name === null || name === undefined || name === "") {
    return false;
  }
  return <span className="ml-3 text-text-muted">{name}</span>;
};

const renderLoading = (): ReactElement => (
  <section className="flex flex-col gap-6">
    <Skeleton size="title" />
    <Skeleton size="row" />
    <Skeleton size="panel" />
  </section>
);

const PathwayHeader = (props: { data: KeggPathwayDetail }): ReactElement => {
  const totalGenes = props.data.kos.reduce((sum, ko) => sum + ko.genes.length, ZERO);
  return (
    <header className="flex flex-col gap-2">
      <h1 className="text-2xl font-semibold text-text">
        <span className="font-mono">{props.data.pathway.id}</span>
        {renderPathwayName(props.data.pathway.name)}
      </h1>
      <p className="text-[13px] text-text-subtle">
        {props.data.kos.length} KEGG {pluralize(props.data.kos.length, "ortholog", "orthologs")} ·{" "}
        {totalGenes} matching {pluralize(totalGenes, "gene", "genes")} in this dataset
      </p>
      <p className="text-[12px]">
        <a
          className="text-text-muted underline-offset-2 hover:underline"
          href={`https://www.kegg.jp/entry/${props.data.pathway.id}`}
          rel="noreferrer"
          target="_blank"
        >
          View on KEGG.jp →
        </a>
      </p>
    </header>
  );
};

const KeggPathwayPage = (): ReactElement => {
  const pathwayId = useValidatedParam("pathwayId", pathwayIdSchema, "");
  const { data, isLoading, error } = useQuery(
    keggPathwayOptions({ path: { pathway_id: pathwayId } }),
  );

  if (pathwayId === "") {
    return <EmptyState description="Missing pathway id." title="KEGG pathway not specified" />;
  }
  if (isLoading) {
    return renderLoading();
  }
  if (error) {
    return <ErrorState detail={String(error)} title={`Could not load ${pathwayId}`} />;
  }
  if (!data) {
    return <EmptyState description={pathwayId} title="Pathway not found" />;
  }

  return (
    <section className="flex flex-col gap-6">
      <a className="text-[12px] text-text-subtle hover:underline" href="/pathways">
        ← Pathways
      </a>
      <PathwayHeader data={data} />
      <KeggPathwayBody data={data} />
    </section>
  );
};

export default KeggPathwayPage;
