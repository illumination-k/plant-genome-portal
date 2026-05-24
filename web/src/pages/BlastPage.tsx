/* eslint-disable jsx-a11y/control-has-associated-label, max-lines, max-lines-per-function, max-statements, no-magic-numbers, no-ternary, no-use-before-define, oxc/no-optional-chaining, oxc/no-rest-spread-properties, prefer-destructuring, react-perf/jsx-no-new-function-as-prop, react/jsx-max-depth, unicorn/no-null */
import {
  blastnJobOptions,
  createBlastnJobMutation,
} from "@/api/client/@tanstack/react-query.gen";
import type {
  AnnotatedHomologyHitResponse,
  BlastnJobResponse,
  CreateBlastnJobError,
} from "@/api/client/types.gen";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import type { ColumnDef, SortingState } from "@tanstack/react-table";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import { useMemo, useState } from "react";
import ErrorState from "@/ui/ErrorState";

const defaultAssemblyAccession = "GCA_037833805.1";
const defaultQuery = "ACGTACGTACGTACGTACGTACGTACGTACGT";

const activeStatuses = new Set(["queued", "running"]);

type BlastForm = {
  assemblyAccession: string;
  evalue: string;
  maxTargetSeqs: string;
  query: string;
  task: string;
};

type BlastHitRowData = AnnotatedHomologyHitResponse & {
  rowId: string;
};

const initialForm: BlastForm = {
  assemblyAccession: defaultAssemblyAccession,
  evalue: "10",
  maxTargetSeqs: "50",
  query: defaultQuery,
  task: "blastn",
};

const formatNumber = (value: number): string => new Intl.NumberFormat("en-US").format(value);

const formatScore = (value: number): string => {
  if (value !== 0 && Math.abs(value) < 0.001) {
    return value.toExponential(2);
  }
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 3 }).format(value);
};

const errorMessage = (error: CreateBlastnJobError | Error | unknown): string | null => {
  if (!error) {
    return null;
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

const regionText = (hit: AnnotatedHomologyHitResponse): string => {
  const subject = hit.hit.subjectRegion;
  return `${hit.hit.sequenceName}:${subject.start}-${subject.end}`;
};

const sortLabel = (sort: false | "asc" | "desc"): string => {
  if (sort === "asc") {
    return " ↑";
  }
  if (sort === "desc") {
    return " ↓";
  }
  return "";
};

const blastHitColumns: Array<ColumnDef<BlastHitRowData>> = [
  {
    accessorFn: (row) => row.hit.sequenceName,
    cell: (info) => (
      <span className="font-mono text-[12px] text-text-muted">{info.getValue<string>()}</span>
    ),
    header: "Subject",
    id: "subject",
  },
  {
    accessorFn: regionText,
    cell: (info) => <BlastRegionCell hit={info.row.original} />,
    header: "Region",
    id: "region",
  },
  {
    accessorFn: (row) => row.hit.percentIdentity,
    cell: (info) => `${formatScore(info.getValue<number>())}%`,
    header: "Identity",
    id: "identity",
  },
  {
    accessorFn: (row) => row.hit.alignmentLength,
    cell: (info) => formatNumber(info.getValue<number>()),
    header: "Length",
    id: "length",
  },
  {
    accessorFn: (row) => row.hit.evalue,
    cell: (info) => (
      <span className="font-mono text-[12px]">{formatScore(info.getValue<number>())}</span>
    ),
    header: "E-value",
    id: "evalue",
  },
  {
    accessorFn: (row) => row.hit.bitScore,
    cell: (info) => formatScore(info.getValue<number>()),
    header: "Bit score",
    id: "bitScore",
  },
  {
    accessorFn: (row) => row.overlappingGeneIds.join(" "),
    cell: (info) => <GeneLinks geneIds={info.row.original.overlappingGeneIds} />,
    enableSorting: false,
    header: "Genes",
    id: "genes",
  },
];

const BlastPage = (): ReactElement => {
  const [form, setForm] = useState<BlastForm>(initialForm);
  const [jobId, setJobId] = useState<string | null>(null);
  const createJob = useMutation(createBlastnJobMutation());
  const jobQuery = useQuery({
    ...blastnJobOptions({ path: { job_id: jobId ?? "" } }),
    enabled: jobId !== null,
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status && activeStatuses.has(status) ? 1500 : false;
    },
  });

  const job = jobQuery.data ?? createJob.data ?? null;
  const isSubmitting = createJob.isPending;
  const isRunning = job ? activeStatuses.has(job.status) : false;
  const error = errorMessage(createJob.error ?? jobQuery.error ?? job?.error);

  const onChange = (
    event: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>,
  ): void => {
    setForm((current) => ({ ...current, [event.target.name]: event.target.value }));
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>): Promise<void> => {
    event.preventDefault();
    const response = await createJob.mutateAsync({
      body: {
        assemblyAccession: form.assemblyAccession.trim(),
        evalue: Number(form.evalue),
        maxTargetSeqs: Number(form.maxTargetSeqs),
        query: form.query,
        task: form.task,
      },
    });
    setJobId(response.id);
  };

  return (
    <section className="grid grid-cols-12 gap-6">
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="text-2xl font-semibold">BLASTN</h2>
            <p className="mt-2 text-sm text-text-muted">
              Marchantia reference homology search
            </p>
          </div>
          {job ? <JobStatusPill job={job} /> : null}
        </div>

        <form className="mt-6 grid grid-cols-12 gap-4" onSubmit={onSubmit}>
          <label className="col-span-12 flex flex-col gap-1 md:col-span-5">
            <span className="text-xs font-medium uppercase text-text-subtle">
              Assembly accession
            </span>
            <input
              className="min-h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="assemblyAccession"
              onChange={onChange}
              required
              value={form.assemblyAccession}
            />
          </label>

          <label className="col-span-12 flex flex-col gap-1 sm:col-span-4 md:col-span-3">
            <span className="text-xs font-medium uppercase text-text-subtle">Task</span>
            <select
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="task"
              onChange={onChange}
              value={form.task}
            >
              <option value="blastn">blastn</option>
              <option value="blastn-short">blastn-short</option>
              <option value="megablast">megablast</option>
              <option value="dc-megablast">dc-megablast</option>
            </select>
          </label>

          <label className="col-span-6 flex flex-col gap-1 sm:col-span-4 md:col-span-2">
            <span className="text-xs font-medium uppercase text-text-subtle">E-value</span>
            <input
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              min="0.0000000001"
              name="evalue"
              onChange={onChange}
              required
              step="any"
              type="number"
              value={form.evalue}
            />
          </label>

          <label className="col-span-6 flex flex-col gap-1 sm:col-span-4 md:col-span-2">
            <span className="text-xs font-medium uppercase text-text-subtle">Max hits</span>
            <input
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              min="1"
              name="maxTargetSeqs"
              onChange={onChange}
              required
              step="1"
              type="number"
              value={form.maxTargetSeqs}
            />
          </label>

          <label className="col-span-12 flex flex-col gap-1">
            <span className="text-xs font-medium uppercase text-text-subtle">Query</span>
            <textarea
              className="min-h-48 resize-y rounded-md border border-border bg-surface px-3 py-3 font-mono text-sm leading-6 text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="query"
              onChange={onChange}
              required
              spellCheck={false}
              value={form.query}
            />
          </label>

          <div className="col-span-12 flex flex-wrap items-center gap-3">
            <button
              className="min-h-10 rounded-md bg-primary-700 px-4 text-sm font-semibold text-white transition hover:bg-primary-800 disabled:cursor-not-allowed disabled:bg-text-disabled"
              disabled={isSubmitting || isRunning}
              type="submit"
            >
              {isSubmitting || isRunning ? "Running" : "Run BLASTN"}
            </button>
            {job ? <span className="font-mono text-xs text-text-muted">{job.id}</span> : null}
          </div>
        </form>
      </div>

      <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
        <div className="border-b border-border-subtle px-6 py-4">
          <h3 className="text-base font-semibold">Results</h3>
        </div>
        <BlastResults error={error} job={job} />
      </div>
    </section>
  );
};

const JobStatusPill = (props: { job: BlastnJobResponse }): ReactElement => (
  <span className="rounded-full border border-border bg-surface-muted px-3 py-1 font-mono text-xs text-text-muted">
    {props.job.status}
  </span>
);

const BlastResults = (props: {
  error: string | null;
  job: BlastnJobResponse | null;
}): ReactElement => {
  if (props.error) {
    return (
      <div className="p-6">
        <ErrorState detail={props.error} title="BLAST job failed" />
      </div>
    );
  }
  if (!props.job) {
    return <p className="px-6 py-8 text-sm text-text-muted">No BLAST job submitted.</p>;
  }
  if (activeStatuses.has(props.job.status)) {
    return <p className="px-6 py-8 text-sm text-text-muted">Job is {props.job.status}.</p>;
  }
  if (!props.job.result || props.job.result.hits.length === 0) {
    return <p className="px-6 py-8 text-sm text-text-muted">No hits found.</p>;
  }
  return <BlastHitTable hits={props.job.result.hits} />;
};

const BlastHitTable = (props: { hits: AnnotatedHomologyHitResponse[] }): ReactElement => {
  const [globalFilter, setGlobalFilter] = useState("");
  const [sorting, setSorting] = useState<SortingState>([]);
  const data = useMemo(
    () =>
      props.hits.map((hit) => ({
        ...hit,
        rowId: [
          hit.hit.queryId,
          hit.hit.sequenceName,
          hit.hit.subjectStart,
          hit.hit.subjectEnd,
          hit.hit.queryStart,
          hit.hit.queryEnd,
          hit.hit.bitScore,
        ].join(":"),
      })),
    [props.hits],
  );
  const table = useReactTable({
    columns: blastHitColumns,
    data,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getRowId: (row) => row.rowId,
    getSortedRowModel: getSortedRowModel(),
    onGlobalFilterChange: setGlobalFilter,
    onSortingChange: setSorting,
    state: {
      globalFilter,
      sorting,
    },
  });
  const rows = table.getRowModel().rows;

  return (
    <div>
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border-subtle px-4 py-3">
        <input
          className="min-h-9 w-full max-w-sm rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          onChange={(event) => setGlobalFilter(event.target.value)}
          placeholder="Filter subject, region, or gene"
          type="search"
          value={globalFilter}
        />
        <span className="text-xs text-text-muted">
          {formatNumber(rows.length)} / {formatNumber(data.length)} hits
        </span>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full min-w-[960px] text-left text-sm">
          <thead className="bg-surface-muted text-xs uppercase text-text-muted">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <th className="px-4 py-3 font-medium" key={header.id}>
                    <button
                      className="inline-flex items-center text-left uppercase disabled:cursor-default"
                      disabled={!header.column.getCanSort()}
                      onClick={header.column.getToggleSortingHandler()}
                      type="button"
                    >
                      {flexRender(header.column.columnDef.header, header.getContext())}
                      {sortLabel(header.column.getIsSorted())}
                    </button>
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody className="divide-y divide-border-subtle">
            {rows.map((row) => (
              <tr className="align-top hover:bg-surface-muted" key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <td className="px-4 py-3 tabular-nums" key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

const BlastRegionCell = (props: { hit: AnnotatedHomologyHitResponse }): ReactElement => {
  const hit = props.hit.hit;
  const region = regionText(props.hit);

  return (
    <div>
      <a
        className="font-mono text-[12px] text-primary-800 hover:underline"
        href={`/browser?loc=${encodeURIComponent(region)}`}
      >
        {region}
      </a>
      <details className="mt-2">
        <summary className="cursor-pointer text-xs text-text-subtle">Alignment</summary>
        <pre className="mt-2 max-w-[40rem] overflow-x-auto rounded-md bg-surface-muted p-3 font-mono text-[11px] leading-5 text-text-muted">
          {`query   ${hit.queryStart}-${hit.queryEnd}   ${hit.queryAlignment}
subject ${hit.subjectStart}-${hit.subjectEnd}   ${hit.subjectAlignment}`}
        </pre>
      </details>
    </div>
  );
};

const GeneLinks = (props: { geneIds: string[] }): ReactElement => {
  if (props.geneIds.length === 0) {
    return <span className="text-xs text-text-subtle">none</span>;
  }
  return (
    <div className="flex flex-wrap gap-2">
      {props.geneIds.map((geneId) => (
        <a
          className="rounded border border-border-subtle bg-surface px-2 py-1 font-mono text-[12px] text-primary-800 hover:border-border hover:underline"
          href={`/genes/${geneId}`}
          key={geneId}
        >
          {geneId}
        </a>
      ))}
    </div>
  );
};

export default BlastPage;
