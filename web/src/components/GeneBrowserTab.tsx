import type { GeneRecord } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneGenomeBrowser from "@/components/GeneGenomeBrowser";

const GeneBrowserTab = (props: { geneRecord: GeneRecord }): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
      <GeneGenomeBrowser gene={props.geneRecord.gene} />
    </div>
  </section>
);

export default GeneBrowserTab;
