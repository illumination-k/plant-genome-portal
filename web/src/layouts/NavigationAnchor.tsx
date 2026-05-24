import type { ReactElement } from "react";
import { useLocation } from "react-router";

const activeLinkClass = "bg-white text-emerald-800 shadow-sm";
const baseLinkClass = "rounded-md px-3 py-2 transition";
const inactiveLinkClass = "text-zinc-600 hover:bg-white hover:text-zinc-950";

const NavigationAnchor = (props: { href: string; label: string }): ReactElement => {
  const location = useLocation();
  let linkClass = `${baseLinkClass} ${inactiveLinkClass}`;

  if (location.pathname === props.href) {
    linkClass = `${baseLinkClass} ${activeLinkClass}`;
  }

  return (
    <a className={linkClass} href={props.href}>
      {props.label}
    </a>
  );
};

export default NavigationAnchor;
