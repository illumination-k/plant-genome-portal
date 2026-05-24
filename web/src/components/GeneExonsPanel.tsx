import type { Exon } from "@/api/client/types.gen";
import ExonContent from "@/components/ExonContent";
import type { ReactElement } from "react";

const GeneExonsPanel = (props: { exons: Exon[] }): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
    <div className="border-b border-border-subtle px-6 py-4">
      <h3 className="text-base font-semibold">Exons</h3>
    </div>
    <ExonContent exons={props.exons} />
  </div>
);

export default GeneExonsPanel;
