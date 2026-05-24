import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneStructureAxis from "@/components/GeneStructureAxis";
import GeneStructureLegend from "@/components/GeneStructureLegend";
import GeneStructureTrack from "@/components/GeneStructureTrack";
import geneStructure from "@/lib/geneStructure";

const GeneStructureViz = (props: { geneRecord: GeneRecord }): ReactElement => {
  const { gene } = props.geneRecord;
  const groups = geneStructure.groupByTranscript(props.geneRecord);
  const scale = geneStructure.makeScale(gene.region.start, gene.region.end);
  const height = geneStructure.totalSvgHeight(groups.length);
  const axisY = geneStructure.computeAxisY(groups.length);

  if (geneStructure.isEmpty(groups.length)) {
    return (
      <div className="mt-4 rounded-md border border-dashed border-border-subtle bg-surface-muted px-4 py-6 text-center text-sm text-text-muted">
        No transcripts to visualize.
      </div>
    );
  }

  return (
    <div className="mt-4">
      <svg
        viewBox={`0 0 ${geneStructure.VIEWBOX_WIDTH} ${height}`}
        preserveAspectRatio="xMidYMid meet"
        className="block h-auto w-full"
        aria-label={`Gene structure of ${gene.id}`}
      >
        {groups.map((group, index) => (
          <GeneStructureTrack
            key={group.transcript.id}
            group={group}
            scale={scale}
            rowIndex={index}
          />
        ))}
        <GeneStructureAxis
          scale={scale}
          start={gene.region.start}
          end={gene.region.end}
          posY={axisY}
        />
      </svg>
      <GeneStructureLegend />
    </div>
  );
};

export default GeneStructureViz;
