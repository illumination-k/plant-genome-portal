import type { ReactElement } from "react";

const TopBarBrand = (): ReactElement => (
  <a className="font-mono text-sm font-semibold tracking-tight text-text" href="/">
    <span className="text-primary-700">▲ </span>plant-genome-portal
  </a>
);

export default TopBarBrand;
