import HeaderBrand from "@/layouts/HeaderBrand";
import PrimaryNavigation from "@/layouts/PrimaryNavigation";
import type { ReactElement } from "react";

const HeaderGrid = (): ReactElement => (
  <div className="mx-auto grid max-w-7xl grid-cols-12 items-center gap-4 px-5 py-4">
    <HeaderBrand />
    <PrimaryNavigation />
  </div>
);

export default HeaderGrid;
