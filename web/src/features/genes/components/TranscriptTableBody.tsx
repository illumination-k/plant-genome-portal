import type { Transcript } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import TranscriptRow from "@/features/genes/components/TranscriptRow";

const emptyCount = 0;

const TranscriptTableBody = (props: {
  exonCounts: Map<string, number>;
  transcripts: Transcript[];
}): ReactElement => (
  <tbody className="divide-y divide-border-subtle">
    {props.transcripts.map((transcript) => (
      <TranscriptRow
        exonCount={props.exonCounts.get(transcript.id) ?? emptyCount}
        key={transcript.id}
        transcript={transcript}
      />
    ))}
  </tbody>
);

export default TranscriptTableBody;
