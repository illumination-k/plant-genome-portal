import AnalysisPanel from "@/features/analysis/components/AnalysisPanel";
import type { ReactElement } from "react";

const AnalysisPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <AnalysisPanel />
  </section>
);

export default AnalysisPage;
