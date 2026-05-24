import type { ReactElement } from "react";

const StatusMessage = (props: { detail: string; title: string }): ReactElement => (
  <div className="rounded-lg border border-dashed border-zinc-300 bg-white p-6">
    <h3 className="text-base font-semibold text-zinc-900">{props.title}</h3>
    <p className="mt-2 text-sm leading-6 text-zinc-600">{props.detail}</p>
  </div>
);

export default StatusMessage;
