import HeaderGrid from "@/layouts/HeaderGrid";
import type { ReactElement } from "react";

const SiteHeader = (): ReactElement => (
  <header className="border-b border-zinc-200 bg-white">
    <HeaderGrid />
  </header>
);

export default SiteHeader;
