import type { ReactElement } from "react";
import GeneStructureCdsBox from "@/components/GeneStructureCdsBox";
import GeneStructureChevron from "@/components/GeneStructureChevron";
import GeneStructureExonBox from "@/components/GeneStructureExonBox";
import geneStructure from "@/lib/geneStructure";

type Group = ReturnType<typeof geneStructure.groupByTranscript>[number];
type Scale = ReturnType<typeof geneStructure.makeScale>;

const GeneStructureTrack = (props: {
  group: Group;
  rowIndex: number;
  scale: Scale;
}): ReactElement => {
  const tokens = geneStructure.trackTokens(props.group, props.scale, props.rowIndex);
  return (
    <g>
      <text
        x={tokens.label.posX}
        y={tokens.label.posY}
        className="fill-text text-[11px] font-medium"
      >
        {tokens.label.id}
        <title>{tokens.label.title}</title>
      </text>
      <line
        x1={tokens.intronLine.x1}
        x2={tokens.intronLine.x2}
        y1={tokens.intronLine.posY}
        y2={tokens.intronLine.posY}
        className="stroke-border"
        strokeWidth={geneStructure.STROKE_INTRON}
      />
      {tokens.chevrons.map((chevron) => (
        <GeneStructureChevron key={chevron.key} pathD={chevron.pathD} />
      ))}
      {tokens.exonBoxes.map((box) => (
        <GeneStructureExonBox
          key={box.key}
          posX={box.posX}
          posY={box.posY}
          width={box.width}
          height={box.height}
          title={box.title}
        />
      ))}
      {tokens.cdsBoxes.map((box) => (
        <GeneStructureCdsBox
          key={box.key}
          posX={box.posX}
          posY={box.posY}
          width={box.width}
          height={box.height}
          title={box.title}
        />
      ))}
    </g>
  );
};

export default GeneStructureTrack;
