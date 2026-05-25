import type { ReactElement } from "react";
import { useLocation } from "react-router";

const baseItem =
  "flex items-center rounded-md px-3 py-1.5 text-sm transition focus-visible:outline-none";
const activeItem = "bg-primary-50 font-medium text-primary-800";
const idleItem = "text-text-muted hover:bg-surface-muted hover:text-text";

const isActiveLink = (currentPath: string, target: string): boolean => {
  if (target === "/") {
    return currentPath === "/";
  }
  return currentPath === target || currentPath.startsWith(`${target}/`);
};

const variantClass = (active: boolean): string => {
  if (active) {
    return activeItem;
  }
  return idleItem;
};

const SideRailLink = (props: { label: string; to: string }): ReactElement => {
  const location = useLocation();
  const active = isActiveLink(location.pathname, props.to);
  const className = `${baseItem} ${variantClass(active)}`;

  return (
    <li>
      <a className={className} href={props.to}>
        {props.label}
      </a>
    </li>
  );
};

export default SideRailLink;
