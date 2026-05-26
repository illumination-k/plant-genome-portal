import type { ReactElement, ReactNode } from "react";

const EMPTY = 0;

const renderEmpty = (label: string): ReactElement => (
  <span className="text-[12px] text-text-subtle">No {label} linked</span>
);

const renderBody = (props: {
  children: ReactNode;
  count: number;
  emptyLabel: string;
}): ReactElement => {
  if (props.count === EMPTY) {
    return renderEmpty(props.emptyLabel);
  }
  return <div className="flex flex-wrap gap-1">{props.children}</div>;
};

const GeneKeggLinkList = (props: {
  children: ReactNode;
  count: number;
  emptyLabel: string;
  label: string;
}): ReactElement => (
  <div>
    <div className="mb-1 text-[11px] uppercase tracking-wide text-text-subtle">{props.label}</div>
    {renderBody(props)}
  </div>
);

export default GeneKeggLinkList;
