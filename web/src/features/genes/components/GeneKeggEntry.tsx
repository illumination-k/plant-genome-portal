import type { GeneKeggOrthologyEntry } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneKeggEntryLinks from "@/features/genes/components/GeneKeggEntryLinks";
import GeneKeggLinkChip from "@/features/genes/components/GeneKeggLinkChip";

const GeneKeggEntry = (props: { entry: GeneKeggOrthologyEntry }): ReactElement => (
  <div className="space-y-2 border-b border-border-subtle pb-3 last:border-b-0 last:pb-0">
    <GeneKeggLinkChip id={props.entry.ko} name={props.entry.name} />
    <GeneKeggEntryLinks entry={props.entry} />
  </div>
);

export default GeneKeggEntry;
