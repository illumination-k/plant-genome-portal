import type { ReactElement } from "react";
import SideRailDisabledLink from "@/layouts/SideRailDisabledLink";
import SideRailLink from "@/layouts/SideRailLink";

type Item = {
  disabled?: boolean;
  label: string;
  to: string;
};

const renderItem = (item: Item): ReactElement => {
  if (item.disabled === true) {
    return <SideRailDisabledLink key={item.to} label={item.label} />;
  }
  return <SideRailLink key={item.to} label={item.label} to={item.to} />;
};

const SideRailGroup = (props: { heading: string; items: Item[] }): ReactElement => (
  <div className="flex flex-col gap-1">
    <h2 className="px-3 text-[11px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
      {props.heading}
    </h2>
    <ul className="flex list-none flex-col gap-0.5 p-0">{props.items.map(renderItem)}</ul>
  </div>
);

export default SideRailGroup;
