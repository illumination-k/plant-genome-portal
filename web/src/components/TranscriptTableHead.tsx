import type { ReactElement } from "react";
import TranscriptHeaderRow from "@/components/TranscriptHeaderRow";

const TranscriptTableHead = (): ReactElement => (
  <thead className="bg-zinc-50 text-zinc-600">
    <TranscriptHeaderRow />
  </thead>
);

export default TranscriptTableHead;
