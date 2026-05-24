import AnalysisText from "@/components/AnalysisText";
import type { ReactElement } from "react";

const AnalysisPanel = (): ReactElement => (
  <div className="col-span-12 rounded-lg border border-zinc-200 bg-white p-6">
    <AnalysisText />
  </div>
);

export default AnalysisPanel;
