import type { ReactElement } from "react";

const StatusMessage = (props: { detail: string; title: string }): ReactElement => (
  <div className="rounded-lg border border-dashed border-border bg-surface p-6">
    <h3 className="text-base font-semibold text-text">{props.title}</h3>
    <p className="mt-2 text-sm leading-6 text-text-muted">{props.detail}</p>
  </div>
);

export default StatusMessage;
