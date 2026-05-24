import type { Transcript } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import TranscriptTableBody from "@/components/TranscriptTableBody";
import TranscriptTableHead from "@/components/TranscriptTableHead";

const TranscriptTable = (props: {
  exonCounts: Map<string, number>;
  transcripts: Transcript[];
}): ReactElement => (
  <div className="overflow-x-auto">
    <table className="w-full min-w-[640px] text-left text-sm">
      <TranscriptTableHead />
      <TranscriptTableBody exonCounts={props.exonCounts} transcripts={props.transcripts} />
    </table>
  </div>
);

export default TranscriptTable;
