/* oxlint-disable no-magic-numbers, jsx-max-depth, jsx-no-new-function-as-prop, max-dependencies, max-lines-per-function, no-ternary */
import { expressionClustergramOptions } from "@/api/client/@tanstack/react-query.gen";
import type { ExpressionClustergramResponse } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import { useMemo, useState } from "react";
import ExpressionClustergram from "@/features/analysis/components/ExpressionClustergram";
import ExpressionLinePlot from "@/features/analysis/components/ExpressionLinePlot";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";
import Tabs from "@/shared/ui/Tabs";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";

const defaultAssemblyAccession = "GCA_037833805.1";
const defaultGeneIds = "Mp1g00070, Mp1g00080, Mp1g00090";
const defaultLimit = "24";

type AnalysisForm = {
  assemblyAccession: string;
  geneIds: string;
  limit: string;
  unit: "tpm" | "fpkm" | "rpkm" | "cpm" | "raw_count" | "normalized_count";
};

const initialForm: AnalysisForm = {
  assemblyAccession: defaultAssemblyAccession,
  geneIds: defaultGeneIds,
  limit: defaultLimit,
  unit: "tpm",
};

const normalizeGeneIds = (value: string): string =>
  value
    .split(/[\s,]+/u)
    .map((geneId) => geneId.trim())
    .filter(Boolean)
    .join(",");

const hasMatrixData = (matrix: ExpressionClustergramResponse | undefined): boolean =>
  Boolean(matrix && matrix.genes.length > 0 && matrix.samples.length > 0 && matrix.values.length > 0);

const AnalysisPanel = (): ReactElement => {
  const [form, setForm] = useState<AnalysisForm>(initialForm);
  const [submitted, setSubmitted] = useState<AnalysisForm>(initialForm);
  const [tab, setTab] = useState("lineplot");
  const query = useQuery(
    expressionClustergramOptions({
      query: {
        assemblyAccession: submitted.assemblyAccession,
        geneIds: normalizeGeneIds(submitted.geneIds),
        limit: Number(submitted.limit),
        unit: submitted.unit,
      },
    }),
  );

  const tabs = useMemo(
    () => [
      {
        label: "Lineplot",
        panel: query.data ? <ExpressionLinePlot matrix={query.data} /> : undefined,
        value: "lineplot",
      },
      {
        label: "Clustergram",
        panel: query.data ? <ExpressionClustergram matrix={query.data} /> : undefined,
        value: "clustergram",
      },
    ],
    [query.data],
  );

  const onChange = (
    event: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>,
  ): void => {
    setForm((current) => ({
      ...current,
      [event.target.name]: event.target.value,
    }));
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    setSubmitted(form);
  };

  return (
    <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className="text-2xl font-semibold text-text">Expression Analysis</h2>
          <p className="mt-2 text-sm leading-6 text-text-muted">
            Compare gene expression profiles across configured RNA-seq runs.
          </p>
        </div>
        <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-xs text-text-muted">
          D3 + Rust
        </span>
      </div>

      <form className="mb-6 grid grid-cols-12 gap-4" onSubmit={onSubmit}>
        <label className="col-span-12 flex flex-col gap-1 text-sm font-medium text-text md:col-span-4">
          Assembly
          <input
            aria-label="Assembly"
            className="rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text"
            name="assemblyAccession"
            onChange={onChange}
            value={form.assemblyAccession}
          />
        </label>
        <label className="col-span-12 flex flex-col gap-1 text-sm font-medium text-text md:col-span-3">
          Unit
          <select
            aria-label="Unit"
            className="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text"
            name="unit"
            onChange={onChange}
            value={form.unit}
          >
            <option value="tpm">TPM</option>
            <option value="fpkm">FPKM</option>
            <option value="rpkm">RPKM</option>
            <option value="cpm">CPM</option>
            <option value="raw_count">Raw count</option>
            <option value="normalized_count">Normalized count</option>
          </select>
        </label>
        <label className="col-span-8 flex flex-col gap-1 text-sm font-medium text-text md:col-span-3">
          Runs
          <input
            aria-label="Runs"
            className="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text"
            min="1"
            name="limit"
            onChange={onChange}
            type="number"
            value={form.limit}
          />
        </label>
        <div className="col-span-4 flex items-end md:col-span-2">
          <button
            className="w-full rounded-md bg-primary-700 px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-primary-800"
            type="submit"
          >
            Plot
          </button>
        </div>
        <label className="col-span-12 flex flex-col gap-1 text-sm font-medium text-text">
          Genes
          <textarea
            aria-label="Genes"
            className="min-h-24 rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text"
            name="geneIds"
            onChange={onChange}
            value={form.geneIds}
          />
        </label>
      </form>

      {query.isLoading && <Skeleton size="panel" />}
      {query.error && (
        <ErrorState
          detail={geneRecordUtils.errorMessage(query.error)}
          title="Expression analysis could not be loaded"
        />
      )}
      {query.data && !hasMatrixData(query.data) && (
        <EmptyState
          description="No expression matrix is available for the selected genes and runs."
          title="No expression data"
        />
      )}
      {hasMatrixData(query.data) && (
        <Tabs ariaLabel="Expression visualizations" onValueChange={setTab} tabs={tabs} value={tab} />
      )}
    </div>
  );
};

export default AnalysisPanel;
