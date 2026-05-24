import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneAttributesPanel from "@/components/GeneAttributesPanel";
import GeneModel from "@/components/GeneModel";

const GeneOverviewTab = (props: { geneRecord: GeneRecord }): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6 lg:col-span-8">
      <h3 className="text-base font-semibold text-text">Gene structure</h3>
      <GeneModel geneRecord={props.geneRecord} />
    </div>
    <div className="col-span-12 lg:col-span-4">
      <GeneAttributesPanel gene={props.geneRecord.gene} />
    </div>
  </section>
);

export default GeneOverviewTab;
