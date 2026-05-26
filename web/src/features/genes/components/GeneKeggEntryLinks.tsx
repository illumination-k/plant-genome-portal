import type { GeneKeggOrthologyEntry } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneKeggLinkChip from "@/features/genes/components/GeneKeggLinkChip";
import GeneKeggLinkList from "@/features/genes/components/GeneKeggLinkList";
import GeneKeggPathwayChip from "@/features/genes/components/GeneKeggPathwayChip";

const renderPathwayChips = (pathways: GeneKeggOrthologyEntry["pathways"]): ReactElement[] =>
  pathways.map((pathway) => (
    <GeneKeggPathwayChip id={pathway.id} key={pathway.id} name={pathway.name} />
  ));

const renderModuleChips = (modules: GeneKeggOrthologyEntry["modules"]): ReactElement[] =>
  modules.map((module) => <GeneKeggLinkChip id={module.id} key={module.id} name={module.name} />);

const renderReactionChips = (reactions: GeneKeggOrthologyEntry["reactions"]): ReactElement[] =>
  reactions.map((reaction) => (
    <GeneKeggLinkChip id={reaction.id} key={reaction.id} name={reaction.name} />
  ));

const GeneKeggEntryLinks = (props: { entry: GeneKeggOrthologyEntry }): ReactElement => (
  <div className="grid grid-cols-1 gap-2 pl-2 text-[12px] md:grid-cols-3">
    <GeneKeggLinkList count={props.entry.pathways.length} emptyLabel="pathways" label="Pathways">
      {renderPathwayChips(props.entry.pathways)}
    </GeneKeggLinkList>
    <GeneKeggLinkList count={props.entry.modules.length} emptyLabel="modules" label="Modules">
      {renderModuleChips(props.entry.modules)}
    </GeneKeggLinkList>
    <GeneKeggLinkList count={props.entry.reactions.length} emptyLabel="reactions" label="Reactions">
      {renderReactionChips(props.entry.reactions)}
    </GeneKeggLinkList>
  </div>
);

export default GeneKeggEntryLinks;
