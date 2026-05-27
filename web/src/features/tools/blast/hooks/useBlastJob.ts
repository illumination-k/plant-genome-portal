/* oxlint-disable import/no-named-export, import/prefer-default-export */
import {
  blastnJobOptions,
  blastpJobOptions,
  createBlastnJobMutation,
  createBlastpJobMutation,
} from "@/api/client/@tanstack/react-query.gen";
import type {
  BlastnJobRequest,
  BlastnJobResponse,
  BlastpJobRequest,
  HomologySearchMethod,
} from "@/api/client/types.gen";
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

type BlastJobQuery = {
  state: {
    data?: BlastnJobResponse;
  };
};

type BlastnJobMutation = {
  mutateAsync: (variables: { body: BlastnJobRequest }) => Promise<BlastnJobResponse>;
};

type BlastpJobMutation = {
  mutateAsync: (variables: { body: BlastpJobRequest }) => Promise<BlastnJobResponse>;
};

type BlastJobMutations = {
  createBlastn: BlastnJobMutation;
  createBlastp: BlastpJobMutation;
};

const refetchActiveJob = (query: BlastJobQuery): number | false => {
  const status = query.state.data?.status;
  if (status && activeStatuses.has(status)) {
    return pollingIntervalMs;
  }
  return false;
};

const queryEnabled = (
  jobId: string | undefined,
  activeMethod: HomologySearchMethod,
  queryMethod: HomologySearchMethod,
): boolean => jobId !== undefined && activeMethod === queryMethod;

const requestBodyFromForm = (form: BlastForm): BlastnJobRequest => ({
  assemblyAccession: form.assemblyAccession.trim(),
  evalue: Number(form.evalue),
  maxTargetSeqs: Number(form.maxTargetSeqs),
  query: form.query,
  task: form.task,
});

const submitBlastJob = (
  method: HomologySearchMethod,
  body: BlastnJobRequest,
  mutations: BlastJobMutations,
): Promise<BlastnJobResponse> => {
  const { createBlastn, createBlastp } = mutations;
  if (method === "blastp") {
    return createBlastp.mutateAsync({ body });
  }
  return createBlastn.mutateAsync({ body });
};

const selectByMethod = <TValue>(
  method: HomologySearchMethod,
  blastn: TValue,
  blastp: TValue,
): TValue => {
  if (method === "blastp") {
    return blastp;
  }
  return blastn;
};

const isRunningJob = (job: BlastnJobResponse | undefined): boolean =>
  job !== undefined && activeStatuses.has(job.status);

export const useBlastJob = (): BlastJobState => {
  const [jobId, setJobId] = useState<string | undefined>();
  const [jobMethod, setJobMethod] = useState<HomologySearchMethod>("blastn");

  const createBlastn = useMutation(createBlastnJobMutation());
  const createBlastp = useMutation(createBlastpJobMutation());
  const blastnQuery = useQuery({
    ...blastnJobOptions({ path: { job_id: jobId ?? "" } }),
    enabled: queryEnabled(jobId, jobMethod, "blastn"),
    refetchInterval: refetchActiveJob,
  });
  const blastpQuery = useQuery({
    ...blastpJobOptions({ path: { job_id: jobId ?? "" } }),
    enabled: queryEnabled(jobId, jobMethod, "blastp"),
    refetchInterval: refetchActiveJob,
  });

  const activeQuery = selectByMethod(jobMethod, blastnQuery, blastpQuery);
  const activeJob = selectByMethod(jobMethod, createBlastn.data, createBlastp.data);
  const job = activeQuery.data ?? activeJob;

  return {
    error: errorMessage(
      createBlastn.error ?? createBlastp.error ?? activeQuery.error ?? job?.error,
    ),
    isRunning: isRunningJob(job),
    isSubmitting: createBlastn.isPending || createBlastp.isPending,
    job,
    jobMethod,
    submit: async (event, form) => {
      event.preventDefault();
      const { method } = form;
      const response = await submitBlastJob(method, requestBodyFromForm(form), {
        createBlastn,
        createBlastp,
      });
      setJobMethod(method);
      setJobId(response.id);
    },
  };
};
