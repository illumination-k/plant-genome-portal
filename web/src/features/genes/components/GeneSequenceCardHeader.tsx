import type { ReactElement } from "react";

const GeneSequenceCardHeader = (props: {
  downloadUrl: string | undefined;
  geneId: string;
}): ReactElement => (
  <header className="flex flex-wrap items-start justify-between gap-3">
    <h3 className="text-base font-semibold text-text">Reference sequence</h3>
    <p className="basis-full text-sm text-text-muted">
      Genomic reference sequence for this gene interval.
    </p>
    {props.downloadUrl && (
      <a
        className="inline-flex min-h-9 items-center rounded-md border border-border bg-surface px-3 text-sm font-medium text-text transition hover:bg-surface-muted"
        download={`${props.geneId}.txt`}
        href={props.downloadUrl}
      >
        Download
      </a>
    )}
  </header>
);

export default GeneSequenceCardHeader;
