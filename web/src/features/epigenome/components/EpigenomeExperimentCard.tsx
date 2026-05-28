/* oxlint-disable no-magic-numbers, no-ternary */
import type { EpigenomeExperimentWithPeaks } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import EpigenomeAssayBadge from "@/features/epigenome/components/EpigenomeAssayBadge";
import EpigenomePeakTable from "@/features/epigenome/components/EpigenomePeakTable";
import EpigenomeTargetBadge from "@/features/epigenome/components/EpigenomeTargetBadge";

const FRIP_PRECISION = 2;

const EpigenomeExperimentCard = (props: { entry: EpigenomeExperimentWithPeaks }): ReactElement => {
  const { experiment, peaks } = props.entry;
  const tissue = experiment.tissue ?? undefined;
  const replicate = experiment.replicate ?? undefined;
  const frip = experiment.frip ?? undefined;
  const peakLabel = peaks.length === 1 ? "peak" : "peaks";

  return (
    <article className="rounded-lg border border-border-subtle bg-surface p-4">
      <header className="mb-3 flex flex-wrap items-center gap-2">
        <span className="font-mono text-xs text-text-muted">{experiment.experimentId}</span>
        <EpigenomeAssayBadge assay={experiment.assay} />
        <EpigenomeTargetBadge target={experiment.target} />
        {tissue !== undefined && <span className="text-xs text-text-muted">tissue: {tissue}</span>}
        {replicate !== undefined && (
          <span className="text-xs text-text-muted">rep: {replicate}</span>
        )}
        {frip !== undefined && (
          <span className="text-xs text-text-muted">FRiP: {frip.toFixed(FRIP_PRECISION)}</span>
        )}
        <span className="ml-auto text-xs text-text-muted">
          {peaks.length} {peakLabel}
        </span>
      </header>
      {peaks.length > 0 && (
        <div className="overflow-hidden rounded-md border border-border-subtle">
          <EpigenomePeakTable peaks={peaks} />
        </div>
      )}
      {peaks.length === 0 && (
        <p className="text-sm text-text-muted">
          No peaks overlap the queried region for this experiment.
        </p>
      )}
    </article>
  );
};

export default EpigenomeExperimentCard;
