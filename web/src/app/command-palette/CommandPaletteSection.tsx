import type { ReactElement, ReactNode } from "react";

const CommandPaletteSection = (props: { children: ReactNode; title: string }): ReactElement => (
  <section className="flex flex-col gap-0.5 py-1">
    <h4 className="px-2 pt-1 text-[10px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
      {props.title}
    </h4>
    {props.children}
  </section>
);

export default CommandPaletteSection;
