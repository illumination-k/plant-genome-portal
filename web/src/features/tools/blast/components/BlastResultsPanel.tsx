/* oxlint-disable eslint/no-use-before-define, react-perf/jsx-no-new-function-as-prop, eslint/prefer-destructuring, eslint/no-ternary, react/jsx-max-depth, eslint/max-lines-per-function */
import type {
  AnnotatedHomologyHitResponse,
  BlastnJobResponse,
  HomologySearchMethod,
} from "@/api/client/types.gen";
import {
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import type { ColumnDef, RowData, SortingState } from "@tanstack/react-table";
import type { ReactElement } from "react";
import { useMemo, useState } from "react";
import ErrorState from "@/shared/ui/ErrorState";
import {
  activeStatuses,
  emptyLength,
  formatNumber,
  formatScore,
} from "@/features/tools/blast/lib/blastConfig";

declare module "@tanstack/react-table" {
  // oxlint-disable-next-line consistent-type-definitions
  interface TableMeta<TData extends RowData> {
    method?: HomologySearchMethod;
  }
}

type BlastHitRowData = AnnotatedHomologyHitResponse & {
  method: HomologySearchMethod;
  rowId: string;
};

const subjectHeader = (method: HomologySearchMethod): string =>
  method === "blastp" ? "Transcript" : "Subject";

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
    header: (info) => subjectHeader(info.table.options.meta?.method ?? "blastn"),
    id: "subject",
  },
  {
    accessorFn: regionText,
    cell: (info) => <BlastRegionCell hit={info.row.original} method={info.row.original.method} />,
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

const BlastResultsPanel = (props: {
  error: string | undefined;
  job: BlastnJobResponse | undefined;
  method: HomologySearchMethod;
}): ReactElement => (
  <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
    <div className="border-b border-border-subtle px-6 py-4">
      <h3 className="text-base font-semibold">Results</h3>
    </div>
    <BlastResults error={props.error} job={props.job} method={props.method} />
  </div>
);

const BlastResults = (props: {
  error: string | undefined;
  job: BlastnJobResponse | undefined;
  method: HomologySearchMethod;
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
  if (!props.job.result || props.job.result.hits.length === emptyLength) {
    return <p className="px-6 py-8 text-sm text-text-muted">No hits found.</p>;
  }
  return <BlastHitTable hits={props.job.result.hits} method={props.method} />;
};

const BlastHitTable = (props: {
  hits: AnnotatedHomologyHitResponse[];
  method: HomologySearchMethod;
}): ReactElement => {
  const [globalFilter, setGlobalFilter] = useState("");
  const [sorting, setSorting] = useState<SortingState>([]);
  const data = useMemo(
    () => props.hits.map((hit) => ({ ...hit, method: props.method, rowId: hitRowId(hit) })),
    [props.hits, props.method],
  );
  const table = useReactTable({
    columns: blastHitColumns,
    data,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getRowId: (row) => row.rowId,
    getSortedRowModel: getSortedRowModel(),
    meta: { method: props.method },
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
          aria-label="Filter BLAST results"
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
                      aria-label={`Sort by ${header.column.id}`}
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

const hitRowId = (hit: AnnotatedHomologyHitResponse): string =>
  [
    hit.hit.queryId,
    hit.hit.sequenceName,
    hit.hit.subjectStart,
    hit.hit.subjectEnd,
    hit.hit.queryStart,
    hit.hit.queryEnd,
    hit.hit.bitScore,
  ].join(":");

const BlastRegionCell = (props: {
  hit: AnnotatedHomologyHitResponse;
  method: HomologySearchMethod;
}): ReactElement => {
  const hit = props.hit.hit;
  const region = regionText(props.hit);
  const regionContent =
    props.method === "blastn" ? (
      <a
        className="font-mono text-[12px] text-primary-800 hover:underline"
        href={`/browser?loc=${encodeURIComponent(region)}`}
      >
        {region}
      </a>
    ) : (
      <span className="font-mono text-[12px] text-text-muted">{region}</span>
    );

  return (
    <div>
      {regionContent}
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
  if (props.geneIds.length === emptyLength) {
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

export default BlastResultsPanel;
