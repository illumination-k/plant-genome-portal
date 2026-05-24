import type { ReactElement } from "react";

const HeaderBrand = (): ReactElement => (
  <div className="col-span-12 sm:col-span-5 lg:col-span-6">
    <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
      Plant Genome Portal
    </p>
    <h1 className="text-xl font-semibold text-zinc-950">Genome workspace</h1>
  </div>
);

export default HeaderBrand;
