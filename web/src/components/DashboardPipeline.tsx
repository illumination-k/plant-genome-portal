import PipelineSteps from "@/components/PipelineSteps";
import type { ReactElement } from "react";

const DashboardPipeline = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6 lg:col-span-4">
    <h2 className="text-base font-semibold">Active pipeline</h2>
    <PipelineSteps />
  </div>
);

export default DashboardPipeline;
