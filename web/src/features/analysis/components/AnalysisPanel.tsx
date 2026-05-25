import AnalysisText from "@/features/analysis/components/AnalysisText";
import type { ReactElement } from "react";

const AnalysisPanel = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
    <AnalysisText />
  </div>
);

export default AnalysisPanel;
