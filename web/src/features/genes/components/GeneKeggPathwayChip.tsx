import type { ReactElement } from "react";

const safeName = (name: string | null | undefined): string => name ?? "";

const renderName = (name: string): ReactElement | false => {
  if (name === "") {
    return false;
  }
  return <span className="text-text-muted">{name}</span>;
};

const GeneKeggPathwayChip = (props: {
  id: string;
  name: string | null | undefined;
}): ReactElement => (
  <a
    className="inline-flex max-w-full items-center gap-1.5 truncate rounded-full border border-border-subtle bg-surface-muted px-2 py-0.5 text-[12px] hover:border-border"
    href={`/kegg/pathway/${props.id}`}
  >
    <span className="font-mono">{props.id}</span>
    {renderName(safeName(props.name))}
  </a>
);

export default GeneKeggPathwayChip;
