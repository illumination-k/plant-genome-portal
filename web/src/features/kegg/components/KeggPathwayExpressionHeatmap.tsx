import { expressionClustergramOptions, geneOptions } from "@/api/client/@tanstack/react-query.gen";
import type { ExpressionClustergramResponse, KeggPathwayDetail } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import { useMemo } from "react";
import ExpressionClustergram from "@/shared/bio/ExpressionClustergram";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";

const sampleLimit = 24;
const unit = "tpm";
const zero = 0;

const hasMatrixData = (
  matrix: ExpressionClustergramResponse | undefined,
): matrix is ExpressionClustergramResponse =>
  Boolean(
    matrix &&
    matrix.genes.length > zero &&
    matrix.samples.length > zero &&
    matrix.values.length > zero,
  );

const pathwayGeneIds = (data: KeggPathwayDetail): string[] => {
  const seen = new Set<string>();
  const ids: string[] = [];
  for (const entry of data.kos) {
    for (const gene of entry.genes) {
      if (!seen.has(gene.id)) {
        seen.add(gene.id);
        ids.push(gene.id);
      }
    }
  }
  return ids;
};

const ExpressionHeatmapHeader = (props: {
  assemblyAccession: string;
  geneCount: number;
}): ReactElement => (
  <header className="mb-4 flex flex-wrap items-baseline justify-between gap-3">
    <div>
      <h2 className="text-base font-semibold text-text">Pathway expression</h2>
      <p className="mt-1 text-[12px] text-text-subtle">
        {props.geneCount} genes · TPM · first {sampleLimit} runs
      </p>
    </div>
    {props.assemblyAccession !== "" && (
      <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-[11px] text-text-muted">
        {props.assemblyAccession}
      </span>
    )}
  </header>
);

type ExpressionState = {
  expressionError: unknown;
  geneCount: number;
  geneError: unknown;
  isLoading: boolean;
  matrix: ExpressionClustergramResponse | undefined;
};

const renderRequestError = (state: ExpressionState): ReactElement | false => {
  if (state.geneError) {
    return (
      <ErrorState
        detail={geneRecordUtils.errorMessage(state.geneError)}
        title="Gene lookup failed"
      />
    );
  }
  if (state.expressionError) {
    return (
      <ErrorState
        detail={geneRecordUtils.errorMessage(state.expressionError)}
        title="Expression heatmap could not be loaded"
      />
    );
  }
  return false;
};

const renderMissingMatrix = (
  matrix: ExpressionClustergramResponse | undefined,
): ReactElement | false => {
  if (!matrix || hasMatrixData(matrix)) {
    return false;
  }
  return (
    <EmptyState
      description="No expression matrix is available for the genes in this pathway."
      title="No expression data"
    />
  );
};

const renderExpressionState = (state: ExpressionState): ReactElement | false => {
  if (state.geneCount === zero) {
    return (
      <EmptyState
        description="This pathway has no matching genes in the current KEGG catalog."
        title="No pathway genes"
      />
    );
  }
  if (state.isLoading) {
    return <Skeleton size="panel" />;
  }
  return renderRequestError(state) || renderMissingMatrix(state.matrix);
};

const KeggPathwayExpressionHeatmap = (props: { data: KeggPathwayDetail }): ReactElement => {
  const geneIds = useMemo(() => pathwayGeneIds(props.data), [props.data]);
  const firstGeneId = geneIds[zero] ?? "";
  const geneQuery = useQuery({
    ...geneOptions({ path: { gene_id: firstGeneId } }),
    enabled: firstGeneId !== "",
  });
  const assemblyAccession = geneQuery.data?.gene.assembly_accession ?? "";
  const expressionQuery = useQuery({
    ...expressionClustergramOptions({
      query: {
        assemblyAccession,
        dropMissingGenes: true,
        geneIds: geneIds.join(","),
        limit: sampleLimit,
        unit,
      },
    }),
    enabled: assemblyAccession !== "" && geneIds.length > zero,
  });

  return (
    <section className="rounded-lg border border-border-subtle bg-surface p-4">
      <ExpressionHeatmapHeader assemblyAccession={assemblyAccession} geneCount={geneIds.length} />
      {renderExpressionState({
        expressionError: expressionQuery.error,
        geneCount: geneIds.length,
        geneError: geneQuery.error,
        isLoading: geneQuery.isLoading || expressionQuery.isLoading,
        matrix: expressionQuery.data,
      })}
      {hasMatrixData(expressionQuery.data) && (
        <ExpressionClustergram matrix={expressionQuery.data} />
      )}
    </section>
  );
};

export default KeggPathwayExpressionHeatmap;
