import type { ReactElement } from "react";

const Metric = (props: { label: string; value: string }): ReactElement => (
  <div className="rounded-lg border border-border-subtle bg-surface-muted p-4">
    <p className="text-sm text-text-muted">{props.label}</p>
    <p className="mt-2 text-2xl font-semibold text-text">{props.value}</p>
  </div>
);

export default Metric;
