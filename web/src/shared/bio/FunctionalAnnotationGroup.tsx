import type { ReactElement, ReactNode } from "react";

const FunctionalAnnotationGroup = (props: {
  children: ReactNode;
  count: number;
  label: string;
}): ReactElement => (
  <section className="flex flex-col gap-2">
    <header className="flex items-baseline gap-2">
      <h4 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
        {props.label}
      </h4>
      <span className="text-[12px] tabular-nums text-text-subtle">{props.count}</span>
    </header>
    <div className="flex flex-wrap gap-1.5">{props.children}</div>
  </section>
);

export default FunctionalAnnotationGroup;
