import type { ReactElement } from "react";
import geneStructure from "@/lib/geneStructure";

type Scale = ReturnType<typeof geneStructure.makeScale>;

const GeneStructureAxis = (props: {
  end: number;
  posY: number;
  scale: Scale;
  start: number;
}): ReactElement => {
  const tokens = geneStructure.axisTokens({
    end: props.end,
    posY: props.posY,
    scale: props.scale,
    start: props.start,
  });
  return (
    <g>
      <line
        x1={tokens.lineX1}
        x2={tokens.lineX2}
        y1={tokens.posY}
        y2={tokens.posY}
        className="stroke-zinc-300"
        strokeWidth={geneStructure.STROKE_TICK}
      />
      {tokens.ticks.map((tick) => (
        <g key={tick.label}>
          <line
            x1={tick.posX}
            x2={tick.posX}
            y1={tokens.posY}
            y2={tokens.posY + geneStructure.TICK_HEIGHT}
            className="stroke-zinc-400"
            strokeWidth={geneStructure.STROKE_TICK}
          />
          <text
            x={tick.posX}
            y={tokens.posY + geneStructure.TICK_HEIGHT + geneStructure.TICK_LABEL_OFFSET}
            textAnchor={tick.anchor}
            className="fill-zinc-500 text-[10px] tabular-nums"
          >
            {tick.label}
          </text>
        </g>
      ))}
    </g>
  );
};

export default GeneStructureAxis;
