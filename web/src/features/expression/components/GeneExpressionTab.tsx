/* oxlint-disable no-magic-numbers, jsx-max-depth */
import { geneExpressionOptions } from "@/api/client/@tanstack/react-query.gen";
import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useQuery } from "@tanstack/react-query";
import GeneExpressionBarPlot from "@/features/expression/components/GeneExpressionBarPlot";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";

const defaultLimit = 24;

const GeneExpressionTab = (props: { geneRecord: GeneRecord }): ReactElement => {
  const expressionQuery = useQuery(
    geneExpressionOptions({
      path: { gene_id: props.geneRecord.gene.id },
      query: { limit: defaultLimit, unit: "tpm" },
    }),
  );

  return (
    <section className="grid grid-cols-12 gap-6">
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="mb-5 flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 className="text-base font-semibold text-text">Expression</h3>
            <p className="mt-1 text-sm text-text-muted">
              RNA-seq abundance across configured sample runs.
            </p>
          </div>
          <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-xs text-text-muted">
            {props.geneRecord.gene.id}
          </span>
        </div>
        {expressionQuery.isLoading && <Skeleton size="panel" />}
        {expressionQuery.error && (
          <ErrorState
            detail={geneRecordUtils.errorMessage(expressionQuery.error)}
            title="Expression values could not be loaded"
          />
        )}
        {expressionQuery.data && expressionQuery.data.length === 0 && (
          <EmptyState
            description="No expression measurements are available for this gene in the configured snapshot."
            title="No expression data"
          />
        )}
        {expressionQuery.data && expressionQuery.data.length > 0 && (
          <GeneExpressionBarPlot points={expressionQuery.data} />
        )}
      </div>
    </section>
  );
};

export default GeneExpressionTab;
