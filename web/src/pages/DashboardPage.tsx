import DashboardGenomeBrowser from "@/components/DashboardGenomeBrowser";
import DashboardHero from "@/components/DashboardHero";
import DashboardPipeline from "@/components/DashboardPipeline";
import type { ReactElement } from "react";

const DashboardPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <DashboardHero />
    <DashboardPipeline />
    <DashboardGenomeBrowser />
  </section>
);

export default DashboardPage;
