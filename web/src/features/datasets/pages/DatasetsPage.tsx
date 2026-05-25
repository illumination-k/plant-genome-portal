import DatasetsPanel from "@/features/datasets/components/DatasetsPanel";
import type { ReactElement } from "react";

const DatasetsPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <DatasetsPanel />
  </section>
);

export default DatasetsPage;
