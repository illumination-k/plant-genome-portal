import type { ReactElement } from "react";
import TopBarActions from "@/layouts/TopBarActions";
import TopBarAssembly from "@/layouts/TopBarAssembly";
import TopBarBrand from "@/layouts/TopBarBrand";
import TopBarSearchTrigger from "@/layouts/TopBarSearchTrigger";

const TopBar = (): ReactElement => (
  <header className="sticky top-0 z-30 h-12 border-b border-border-subtle bg-surface">
    <div className="mx-auto flex h-full max-w-[1440px] items-center gap-4 px-6 md:px-8">
      <TopBarBrand />
      <TopBarAssembly />
      <TopBarSearchTrigger />
      <TopBarActions />
    </div>
  </header>
);

export default TopBar;
