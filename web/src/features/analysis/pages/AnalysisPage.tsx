import AnalysisPanel from "@/features/analysis/components/AnalysisPanel";
import EnrichmentAnalysisPanel from "@/features/analysis/components/EnrichmentAnalysisPanel";
import type { ReactElement } from "react";

const AnalysisPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <EnrichmentAnalysisPanel />
    <AnalysisPanel />
  </section>
);

export default AnalysisPage;
