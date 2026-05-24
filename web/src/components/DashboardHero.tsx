import DashboardHeroHeader from "@/components/DashboardHeroHeader";
import DashboardMetricGrid from "@/components/DashboardMetricGrid";
import type { ReactElement } from "react";

const DashboardHero = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6 lg:col-span-8">
    <DashboardHeroHeader />
    <DashboardMetricGrid />
  </div>
);

export default DashboardHero;
