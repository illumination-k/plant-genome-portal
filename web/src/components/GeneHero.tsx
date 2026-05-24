import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneGenomeBrowser from "@/components/GeneGenomeBrowser";
import GeneHeroHeader from "@/components/GeneHeroHeader";
import GeneMetricGrid from "@/components/GeneMetricGrid";
import GeneModel from "@/components/GeneModel";

const GeneHero = (props: { geneRecord: GeneRecord }): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6">
    <GeneHeroHeader gene={props.geneRecord.gene} />
    <GeneMetricGrid geneRecord={props.geneRecord} />
    <GeneModel geneRecord={props.geneRecord} />
    <GeneGenomeBrowser gene={props.geneRecord.gene} />
  </div>
);

export default GeneHero;
