import type { Gene } from "@/api/client/types.gen";
import GeneAttributeList from "@/components/GeneAttributeList";
import type { ReactElement } from "react";

const GeneAttributesPanel = (props: { gene: Gene }): ReactElement => (
  <div className="rounded-lg border border-border-subtle bg-surface p-6">
    <h3 className="text-base font-semibold text-text">Attributes</h3>
    <GeneAttributeList attributes={props.gene.attributes} />
  </div>
);

export default GeneAttributesPanel;
