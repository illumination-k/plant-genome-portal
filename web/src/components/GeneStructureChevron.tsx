import type { ReactElement } from "react";
import geneStructure from "@/lib/geneStructure";

const GeneStructureChevron = (props: { pathD: string }): ReactElement => (
  <path
    d={props.pathD}
    className="stroke-zinc-400"
    fill="none"
    strokeWidth={geneStructure.STROKE_CHEVRON}
    strokeLinecap="round"
    strokeLinejoin="round"
  />
);

export default GeneStructureChevron;
