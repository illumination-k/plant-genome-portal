import PipelineStep from "@/components/PipelineStep";
import type { ReactElement } from "react";

const steps = ["Import FASTA", "Index features", "Publish dataset"];
const oneBasedOffset = 1;

const PipelineSteps = (): ReactElement => (
  <div className="mt-4 space-y-4">
    {steps.map((step, index) => (
      <PipelineStep key={step} label={step} value={String(index + oneBasedOffset)} />
    ))}
  </div>
);

export default PipelineSteps;
