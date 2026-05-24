import type { ReactElement } from "react";

const className =
  "flex cursor-not-allowed items-center rounded-md px-3 py-1.5 text-sm text-text-disabled";

const SideRailDisabledLink = (props: { label: string }): ReactElement => (
  <li>
    <span aria-disabled="true" className={className} title="Coming soon">
      {props.label}
      <span className="ml-auto text-[10px] uppercase tracking-wider text-text-disabled">
        Soon
      </span>
    </span>
  </li>
);

export default SideRailDisabledLink;
