import type { ReactElement, ReactNode } from "react";

const KbdKey = (props: { children: ReactNode }): ReactElement => (
  <kbd className="rounded border border-border-subtle bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] text-text-muted">
    {props.children}
  </kbd>
);

export default KbdKey;
