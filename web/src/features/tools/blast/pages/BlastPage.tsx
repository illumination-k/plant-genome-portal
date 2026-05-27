import type { ReactElement } from "react";
import { useState } from "react";
import {
  BlastFormPanel,
  methodDescription,
  methodLabel,
} from "@/features/tools/blast/components/BlastFormPanel";
import BlastResultsPanel from "@/features/tools/blast/components/BlastResultsPanel";
import { useBlastJob } from "@/features/tools/blast/hooks/useBlastJob";
import { initialForm } from "@/features/tools/blast/lib/blastConfig";
import type { BlastForm } from "@/features/tools/blast/lib/blastConfig";

const BlastPage = (): ReactElement => {
  const [form, setForm] = useState<BlastForm>(initialForm);
  const blastJob = useBlastJob();

  return (
    <section className="grid grid-cols-12 gap-6">
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="text-2xl font-semibold">{methodLabel(form.method)}</h2>
            <p className="mt-2 text-sm text-text-muted">{methodDescription(form.method)}</p>
          </div>
          {blastJob.job && <JobStatusPill jobId={blastJob.job.id} status={blastJob.job.status} />}
        </div>

        <BlastFormPanel
          form={form}
          isRunning={blastJob.isRunning}
          isSubmitting={blastJob.isSubmitting}
          onChange={setForm}
          onSubmit={(event) => blastJob.submit(event, form)}
        />
      </div>

      <BlastResultsPanel error={blastJob.error} job={blastJob.job} method={blastJob.jobMethod} />
    </section>
  );
};

const JobStatusPill = (props: { jobId: string; status: string }): ReactElement => (
  <div className="flex flex-wrap items-center gap-3">
    <span className="rounded-full border border-border bg-surface-muted px-3 py-1 font-mono text-xs text-text-muted">
      {props.status}
    </span>
    <span className="font-mono text-xs text-text-muted">{props.jobId}</span>
  </div>
);

export default BlastPage;
