import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import TranscriptTable from "@/components/TranscriptTable";
import geneRecordUtils from "@/lib/geneRecordUtils";

const GeneTranscriptPanel = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface lg:col-span-7">
    <div className="border-b border-border-subtle px-6 py-4">
      <h3 className="text-base font-semibold">Transcripts</h3>
    </div>
    <TranscriptTable
      exonCounts={geneRecordUtils.countExonsByTranscript(props.geneRecord.exons)}
      transcripts={props.geneRecord.transcripts}
    />
  </div>
);

export default GeneTranscriptPanel;
