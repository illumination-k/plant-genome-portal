/* oxlint-disable import/no-named-export, import/prefer-default-export, eslint/max-statements, eslint/max-lines-per-function, eslint/no-ternary, eslint/prefer-destructuring */
import {
  blastnJobOptions,
  blastpJobOptions,
  createBlastnJobMutation,
  createBlastpJobMutation,
} from "@/api/client/@tanstack/react-query.gen";
import type { BlastnJobResponse, HomologySearchMethod } from "@/api/client/types.gen";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import type { FormEvent } from "react";
import {
  activeStatuses,
  errorMessage,
  pollingIntervalMs,
} from "@/features/tools/blast/lib/blastConfig";
import type { BlastForm } from "@/features/tools/blast/lib/blastConfig";

type BlastJobState = {
  error: string | undefined;
  isRunning: boolean;
  isSubmitting: boolean;
  job: BlastnJobResponse | undefined;
  jobMethod: HomologySearchMethod;
  submit: (event: FormEvent<HTMLFormElement>, form: BlastForm) => Promise<void>;
};

export const useBlastJob = (): BlastJobState => {
  const [jobId, setJobId] = useState<string | undefined>();
  const [jobMethod, setJobMethod] = useState<HomologySearchMethod>("blastn");

  const createBlastn = useMutation(createBlastnJobMutation());
  const createBlastp = useMutation(createBlastpJobMutation());
  const blastnQuery = useQuery({
    ...blastnJobOptions({ path: { job_id: jobId ?? "" } }),
    enabled: jobId !== undefined && jobMethod === "blastn",
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status && activeStatuses.has(status) ? pollingIntervalMs : false;
    },
  });
  const blastpQuery = useQuery({
    ...blastpJobOptions({ path: { job_id: jobId ?? "" } }),
    enabled: jobId !== undefined && jobMethod === "blastp",
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status && activeStatuses.has(status) ? pollingIntervalMs : false;
    },
  });

  const activeQuery = jobMethod === "blastp" ? blastpQuery : blastnQuery;
  const activeMutation = jobMethod === "blastp" ? createBlastp : createBlastn;
  const job = activeQuery.data ?? activeMutation.data;
  const isRunning = job ? activeStatuses.has(job.status) : false;

  return {
    error: errorMessage(
      createBlastn.error ?? createBlastp.error ?? activeQuery.error ?? job?.error,
    ),
    isRunning,
    isSubmitting: createBlastn.isPending || createBlastp.isPending,
    job,
    jobMethod,
    submit: async (event, form) => {
      event.preventDefault();
      const body = {
        assemblyAccession: form.assemblyAccession.trim(),
        evalue: Number(form.evalue),
        maxTargetSeqs: Number(form.maxTargetSeqs),
        query: form.query,
        task: form.task,
      };
      const method = form.method;
      const response =
        method === "blastp"
          ? await createBlastp.mutateAsync({ body })
          : await createBlastn.mutateAsync({ body });
      setJobMethod(method);
      setJobId(response.id);
    },
  };
};
