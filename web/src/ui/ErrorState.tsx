import type { ReactElement } from "react";

const ErrorState = (props: { detail: string; title: string }): ReactElement => (
  <div
    className="rounded-lg border border-dashed border-danger/40 bg-surface px-6 py-6"
    role="alert"
  >
    <h3 className="text-base font-semibold text-danger">{props.title}</h3>
    <p className="mt-2 text-sm text-text-muted">{props.detail}</p>
  </div>
);

export default ErrorState;
