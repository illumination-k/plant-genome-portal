import type { ReactElement } from "react";
import TranscriptHeaderRow from "@/components/TranscriptHeaderRow";

const TranscriptTableHead = (): ReactElement => (
  <thead className="bg-surface-muted text-text-muted">
    <TranscriptHeaderRow />
  </thead>
);

export default TranscriptTableHead;
