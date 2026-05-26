/* oxlint-disable import/no-named-export, import/group-exports, eslint/no-magic-numbers, eslint/max-statements, eslint/no-ternary */
import type {
  CreateBlastnJobError,
  CreateBlastpJobError,
  HomologySearchMethod,
} from "@/api/client/types.gen";

export const defaultAssemblyAccession = "GCA_037833805.1";
export const defaultBlastnQuery = "ACGTACGTACGTACGTACGTACGTACGTACGT";
export const defaultBlastpQuery = "MVTAGSMMHLERMGSELKCPVCLSLYKSAATISCNHTFCRSCILESVRATSCCPICKAHT";
export const pollingIntervalMs = 1500;
export const emptyLength = 0;

export const activeStatuses = new Set(["queued", "running"]);
export const blastnTasks = ["blastn", "blastn-short", "megablast", "dc-megablast"] as const;
export const blastpTasks = ["blastp", "blastp-short", "blastp-fast"] as const;

export type BlastForm = {
  assemblyAccession: string;
  evalue: string;
  maxTargetSeqs: string;
  method: HomologySearchMethod;
  query: string;
  task: string;
};

export const initialForm: BlastForm = {
  assemblyAccession: defaultAssemblyAccession,
  evalue: "10",
  maxTargetSeqs: "50",
  method: "blastn",
  query: defaultBlastnQuery,
  task: "blastn",
};

export const methodDefaults = (method: HomologySearchMethod): { query: string; task: string } => {
  if (method === "blastp") {
    return { query: defaultBlastpQuery, task: "blastp" };
  }
  return { query: defaultBlastnQuery, task: "blastn" };
};

export const methodLabel = (method: HomologySearchMethod): string =>
  method === "blastp" ? "BLASTP" : "BLASTN";

export const methodDescription = (method: HomologySearchMethod): string =>
  method === "blastp"
    ? "Marchantia protein homology search"
    : "Marchantia reference genome homology search";

export const formatNumber = (value: number): string => new Intl.NumberFormat("en-US").format(value);

export const formatScore = (value: number): string => {
  if (value !== emptyLength && Math.abs(value) < 0.001) {
    return value.toExponential(2);
  }
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 3 }).format(value);
};

export const errorMessage = (
  error: CreateBlastnJobError | CreateBlastpJobError | Error | string | unknown,
): string | undefined => {
  if (!error) {
    return undefined;
  }
  if (typeof error === "string") {
    return error;
  }
  if (typeof error === "object" && "error" in error) {
    const value = error.error;
    if (typeof value === "string") {
      return value;
    }
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "BLAST job failed";
};
