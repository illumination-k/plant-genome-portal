import type { ReactElement } from "react";
import geneStructure from "@/shared/lib/geneStructure";

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
    className="fill-primary-200 stroke-primary-700"
    strokeWidth={geneStructure.STROKE_EXON}
  >
    <title>{props.title}</title>
  </rect>
);

export default GeneStructureExonBox;
