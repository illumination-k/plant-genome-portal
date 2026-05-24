import type { Gene } from "@/api/client/types.gen";
import GeneAttributeList from "@/components/GeneAttributeList";
import type { ReactElement } from "react";

const GeneAttributesPanel = (props: { gene: Gene }): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6 lg:col-span-5">
    <h3 className="text-base font-semibold">Attributes</h3>
    <GeneAttributeList attributes={props.gene.attributes} />
  </div>
);

export default GeneAttributesPanel;
