import type { ReactElement } from "react";

const PipelineStep = (props: { label: string; value: string }): ReactElement => (
  <div className="flex items-center gap-3">
    <div className="grid size-8 place-items-center rounded-full bg-emerald-100 text-sm font-semibold text-emerald-800">
      {props.value}
    </div>
    <span className="text-sm text-zinc-700">{props.label}</span>
  </div>
);

export default PipelineStep;
