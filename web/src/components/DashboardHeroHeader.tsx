import DashboardHeroText from "@/components/DashboardHeroText";
import type { ReactElement } from "react";

const DashboardHeroHeader = (): ReactElement => (
  <div className="flex flex-wrap items-start justify-between gap-4">
    <DashboardHeroText />
    <a
      className="rounded-md bg-emerald-700 px-4 py-2 text-sm font-semibold text-white transition hover:bg-emerald-800 focus:outline-none focus:ring-2 focus:ring-emerald-600 focus:ring-offset-2"
      href="/genes"
    >
      New query
    </a>
  </div>
);

export default DashboardHeroHeader;
