import Metric from "@/components/Metric";
import type { ReactElement } from "react";

const DashboardMetricGrid = (): ReactElement => (
  <div className="mt-6 grid gap-4 sm:grid-cols-3">
    <Metric label="Assemblies" value="128" />
    <Metric label="Species" value="42" />
    <Metric label="Annotations" value="316k" />
  </div>
);

export default DashboardMetricGrid;
