import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneExonsPanel from "@/components/GeneExonsPanel";
import GeneTranscriptPanel from "@/components/GeneTranscriptPanel";

const GeneTranscriptsTab = (props: { geneRecord: GeneRecord }): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <GeneTranscriptPanel geneRecord={props.geneRecord} />
    <GeneExonsPanel exons={props.geneRecord.exons} />
  </section>
);

export default GeneTranscriptsTab;
