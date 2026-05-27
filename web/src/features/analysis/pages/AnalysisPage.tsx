import EnrichmentPanel from "@/features/tools/enrichment/components/EnrichmentPanel";
import MultiGeneExpressionPanel from "@/features/tools/multi-gene-expression/components/MultiGeneExpressionPanel";
import type { ReactElement } from "react";

const AnalysisPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <EnrichmentPanel />
    <MultiGeneExpressionPanel />
  </section>
);

export default AnalysisPage;
