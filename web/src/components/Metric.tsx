import type { ReactElement } from "react";

const Metric = (props: { label: string; value: string }): ReactElement => (
  <div className="rounded-lg border border-zinc-200 bg-zinc-50 p-4">
    <p className="text-sm text-zinc-600">{props.label}</p>
    <p className="mt-2 text-2xl font-semibold text-zinc-950">{props.value}</p>
  </div>
);

export default Metric;
