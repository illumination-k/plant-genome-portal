import type { Assay } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const ASSAY_LABEL: Record<Assay, string> = {
  atac_seq: "ATAC-seq",
  chip_seq: "ChIP-seq",
};

const EpigenomeAssayBadge = (props: { assay: Assay }): ReactElement => (
  <span className="inline-flex items-center rounded-md border border-border-subtle bg-surface-muted px-2 py-0.5 text-xs font-medium text-text">
    {ASSAY_LABEL[props.assay]}
  </span>
);

export default EpigenomeAssayBadge;
