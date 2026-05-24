import type { ReactElement } from "react";

const EmptyState = (props: { description: string; title: string }): ReactElement => (
  <div className="rounded-lg border border-dashed border-border bg-surface px-6 py-8 text-center">
    <h3 className="text-base font-semibold text-text">{props.title}</h3>
    <p className="mx-auto mt-2 max-w-md text-sm text-text-muted">{props.description}</p>
  </div>
);

export default EmptyState;
