import type { ReactElement } from "react";
import FunctionalAnnotationChip from "@/bio/FunctionalAnnotationChip";
import FunctionalAnnotationGroup from "@/bio/FunctionalAnnotationGroup";

type Entry = {
  href: string;
  id: string;
  name: string;
};

const EMPTY_ENTRIES = 0;

const renderEmpty = (): ReactElement => (
  <span className="text-[12px] text-text-subtle">—</span>
);

const renderChip = (label: string, entry: Entry): ReactElement => (
  <FunctionalAnnotationChip
    href={entry.href}
    id={entry.id}
    key={`${label}-${entry.id}`}
    name={entry.name}
  />
);

const renderEntries = (label: string, entries: Entry[]): ReactElement | ReactElement[] => {
  if (entries.length === EMPTY_ENTRIES) {
    return renderEmpty();
  }
  return entries.map((entry) => renderChip(label, entry));
};

const GeneAnnotationGroupCard = (props: {
  entries: Entry[];
  label: string;
}): ReactElement => (
  <div className="rounded-lg border border-border-subtle bg-surface p-5">
    <FunctionalAnnotationGroup count={props.entries.length} label={props.label}>
      {renderEntries(props.label, props.entries)}
    </FunctionalAnnotationGroup>
  </div>
);

export default GeneAnnotationGroupCard;
