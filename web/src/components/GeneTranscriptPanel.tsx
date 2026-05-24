import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import TranscriptTable from "@/components/TranscriptTable";
import geneRecordUtils from "@/lib/geneRecordUtils";

const GeneTranscriptPanel = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-zinc-200 bg-white lg:col-span-7">
    <div className="border-b border-zinc-200 px-6 py-4">
      <h3 className="text-base font-semibold">Transcripts</h3>
    </div>
    <TranscriptTable
      exonCounts={geneRecordUtils.countExonsByTranscript(props.geneRecord.exons)}
      transcripts={props.geneRecord.transcripts}
    />
  </div>
);

export default GeneTranscriptPanel;
