/* oxlint-disable no-magic-numbers, jsx-max-depth, max-lines-per-function */
import { geneEpigenomeOptions } from "@/api/client/@tanstack/react-query.gen";
import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useQuery } from "@tanstack/react-query";
import EpigenomeExperimentCard from "@/features/epigenome/components/EpigenomeExperimentCard";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";

const DEFAULT_UPSTREAM_BP = 2000;
const DEFAULT_DOWNSTREAM_BP = 0;

const GeneEpigenomeTab = (props: { geneRecord: GeneRecord }): ReactElement => {
  const epigenomeQuery = useQuery(
    geneEpigenomeOptions({
      path: { gene_id: props.geneRecord.gene.id },
      query: { downstreamBp: DEFAULT_DOWNSTREAM_BP, upstreamBp: DEFAULT_UPSTREAM_BP },
    }),
  );

  return (
    <section className="grid grid-cols-12 gap-6">
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="mb-5 flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 className="text-base font-semibold text-text">Epigenome</h3>
            <p className="mt-1 text-sm text-text-muted">
              ChIP-seq and ATAC-seq experiments with peaks overlapping this gene plus{" "}
              {DEFAULT_UPSTREAM_BP.toLocaleString()} bp upstream of the TSS.
            </p>
          </div>
          {epigenomeQuery.data && (
            <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-xs text-text-muted">
              {epigenomeQuery.data.region.sequenceName}:
              {epigenomeQuery.data.region.start.toLocaleString()}-
              {epigenomeQuery.data.region.end.toLocaleString()}
            </span>
          )}
        </div>
        {epigenomeQuery.isLoading && <Skeleton size="panel" />}
        {epigenomeQuery.error && (
          <ErrorState
            detail={geneRecordUtils.errorMessage(epigenomeQuery.error)}
            title="Epigenome data could not be loaded"
          />
        )}
        {epigenomeQuery.data && epigenomeQuery.data.experiments.length === 0 && (
          <EmptyState
            description="No ChIP-seq or ATAC-seq experiments have peaks overlapping this gene's body or promoter window in the configured snapshot."
            title="No epigenome peaks"
          />
        )}
        {epigenomeQuery.data && epigenomeQuery.data.experiments.length > 0 && (
          <div className="grid grid-cols-1 gap-4">
            {epigenomeQuery.data.experiments.map((entry) => (
              <EpigenomeExperimentCard entry={entry} key={entry.experiment.experimentId} />
            ))}
          </div>
        )}
      </div>
    </section>
  );
};

export default GeneEpigenomeTab;
