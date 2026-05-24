import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneSequenceCard from "@/components/GeneSequenceCard";

const oneBasedOffset = 1;

const GeneSequenceTab = (props: { geneRecord: GeneRecord }): ReactElement => {
  const { region, sequence_name: chr } = props.geneRecord.gene;
  const start = region.start + oneBasedOffset;
  const { end } = region;
  const length = end - region.start;

  return (
    <section className="grid grid-cols-12 gap-6">
      <GeneSequenceCard chr={chr} end={end} length={length} start={start} />
    </section>
  );
};

export default GeneSequenceTab;
