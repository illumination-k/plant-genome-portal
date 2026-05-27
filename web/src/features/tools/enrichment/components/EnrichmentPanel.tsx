/* oxlint-disable no-magic-numbers, jsx-max-depth, jsx-no-new-function-as-prop, max-lines-per-function, no-ternary */
import { enrichmentAnalysisMutation } from "@/api/client/@tanstack/react-query.gen";
import type { EnrichmentAnnotationKind } from "@/api/client/types.gen";
import EnrichmentDownloadLinks from "@/features/tools/enrichment/components/EnrichmentDownloadLinks";
import EnrichmentResultsTable from "@/features/tools/enrichment/components/EnrichmentResultsTable";
import geneRecordUtils from "@/shared/lib/geneRecordUtils";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import { useMutation } from "@tanstack/react-query";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import { useState } from "react";

const defaultAssemblyAccession = "GCA_037833805.1";
const defaultGeneIds = "Mp1g00070, Mp1g00080, Mp1g00090";

type EnrichmentForm = {
  assemblyAccession: string;
  backgroundGeneIds: string;
  geneIds: string;
  limit: string;
  minPopulationHits: string;
};

const initialForm: EnrichmentForm = {
  assemblyAccession: defaultAssemblyAccession,
  backgroundGeneIds: "",
  geneIds: defaultGeneIds,
  limit: "25",
  minPopulationHits: "2",
};

const annotationKindOptions: Array<{ label: string; value: EnrichmentAnnotationKind }> = [
  { label: "GO", value: "go_term" },
  { label: "Pfam", value: "pfam" },
  { label: "InterPro", value: "inter_pro" },
  { label: "KEGG", value: "kegg" },
  { label: "KOG", value: "kog" },
  { label: "NCBIfam", value: "ncbi_fam" },
];

const splitGeneIds = (value: string): string[] =>
  value
    .split(/[\s,]+/u)
    .map((geneId) => geneId.trim())
    .filter(Boolean);

const EnrichmentPanel = (): ReactElement => {
  const [form, setForm] = useState<EnrichmentForm>(initialForm);
  const [annotationKinds, setAnnotationKinds] = useState<EnrichmentAnnotationKind[]>([
    "go_term",
    "pfam",
    "inter_pro",
    "kegg",
  ]);
  const mutation = useMutation(enrichmentAnalysisMutation());

  const onChange = (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>): void => {
    setForm((current) => ({ ...current, [event.target.name]: event.target.value }));
  };

  const onKindChange = (event: ChangeEvent<HTMLInputElement>): void => {
    const value = event.target.value as EnrichmentAnnotationKind;
    setAnnotationKinds((current) =>
      event.target.checked ? [...current, value] : current.filter((kind) => kind !== value),
    );
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    const backgroundGeneIds = splitGeneIds(form.backgroundGeneIds);
    const body = {
      annotationKinds,
      assemblyAccession: form.assemblyAccession.trim(),
      geneIds: splitGeneIds(form.geneIds),
      limit: Number(form.limit),
      minPopulationHits: Number(form.minPopulationHits),
    };
    if (backgroundGeneIds.length > 0) {
      Object.assign(body, { backgroundGeneIds });
    }
    mutation.mutate({ body });
  };

  const result = mutation.data;

  return (
    <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className="text-2xl font-semibold text-text">Enrichment Analysis</h2>
          <p className="mt-2 text-sm leading-6 text-text-muted">
            Run over-representation analysis against functional annotations in the current genome.
          </p>
        </div>
        <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1 font-mono text-xs text-text-muted">
          Fisher + BH-FDR
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
        <label className="col-span-6 flex flex-col gap-1 text-sm font-medium text-text md:col-span-2">
          Min hits
          <input
            aria-label="Minimum population hits"
            className="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text"
            min="1"
            name="minPopulationHits"
            onChange={onChange}
            type="number"
            value={form.minPopulationHits}
          />
        </label>
        <label className="col-span-6 flex flex-col gap-1 text-sm font-medium text-text md:col-span-2">
          Results
          <input
            aria-label="Result limit"
            className="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text"
            min="1"
            name="limit"
            onChange={onChange}
            type="number"
            value={form.limit}
          />
        </label>
        <div className="col-span-12 flex items-end md:col-span-4">
          <button
            className="w-full rounded-md bg-primary-700 px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-primary-800 disabled:cursor-not-allowed disabled:opacity-60"
            disabled={mutation.isPending}
            type="submit"
          >
            {mutation.isPending ? "Running" : "Run enrichment"}
          </button>
        </div>

        <fieldset className="col-span-12 flex flex-wrap gap-3 rounded-md border border-border-subtle px-3 py-2">
          <legend className="px-1 text-sm font-medium text-text">Annotation types</legend>
          {annotationKindOptions.map((option) => (
            <label className="flex items-center gap-2 text-sm text-text-muted" key={option.value}>
              <input
                aria-label={option.label}
                checked={annotationKinds.includes(option.value)}
                className="size-4 accent-primary-700"
                onChange={onKindChange}
                type="checkbox"
                value={option.value}
              />
              {option.label}
            </label>
          ))}
        </fieldset>

        <label className="col-span-12 flex flex-col gap-1 text-sm font-medium text-text md:col-span-6">
          Study genes
          <textarea
            aria-label="Study genes"
            className="min-h-28 rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text"
            name="geneIds"
            onChange={onChange}
            value={form.geneIds}
          />
        </label>
        <label className="col-span-12 flex flex-col gap-1 text-sm font-medium text-text md:col-span-6">
          Background genes
          <textarea
            aria-label="Background genes"
            className="min-h-28 rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text"
            name="backgroundGeneIds"
            onChange={onChange}
            placeholder="Leave blank to use all genes in the assembly"
            value={form.backgroundGeneIds}
          />
        </label>
      </form>

      {mutation.error && (
        <ErrorState
          detail={geneRecordUtils.errorMessage(mutation.error)}
          title="Enrichment analysis could not be run"
        />
      )}

      {result && (
        <div className="space-y-4">
          <div className="flex flex-wrap gap-2 text-xs text-text-muted">
            <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1">
              Study {result.studySize}
            </span>
            <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1">
              Population {result.populationSize}
            </span>
            <span className="rounded-md border border-border-subtle bg-surface-muted px-2 py-1">
              Tested terms {result.testedTerms}
            </span>
            <EnrichmentDownloadLinks result={result} />
          </div>
          {result.results.length === 0 ? (
            <EmptyState
              description="No annotation terms passed the current filters."
              title="No enriched terms"
            />
          ) : (
            <EnrichmentResultsTable results={result.results} />
          )}
        </div>
      )}
    </div>
  );
};

export default EnrichmentPanel;
