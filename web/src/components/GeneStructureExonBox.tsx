import type { ReactElement } from "react";
import geneStructure from "@/lib/geneStructure";

const GeneStructureExonBox = (props: {
  height: number;
  posX: number;
  posY: number;
  title: string;
  width: number;
}): ReactElement => (
  <rect
    x={props.posX}
    y={props.posY}
    width={props.width}
    height={props.height}
    rx={geneStructure.RECT_RADIUS}
    className="fill-emerald-200 stroke-emerald-700"
    strokeWidth={geneStructure.STROKE_EXON}
  >
    <title>{props.title}</title>
  </rect>
);

export default GeneStructureExonBox;
