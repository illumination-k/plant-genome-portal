import {
  refgetSequenceOptions,
  sequenceSegmentsOptions,
} from "@/api/client/@tanstack/react-query.gen";
import type { Strand } from "@/api/client";
import { useQuery } from "@tanstack/react-query";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import { useState } from "react";
import sequenceSegmentsUrl from "@/shared/lib/sequenceSegmentsUrl";
import ErrorState from "@/shared/ui/ErrorState";

const sequenceLineLength = 60;

type FetchForm = {
  checksum: string;
  end: string;
  start: string;
};

type RefgetRequest = {
  checksum: string;
  end?: number;
  start?: number;
};

type SegmentFetchForm = {
  assemblyAccession: string;
  ends: string;
  sequenceName: string;
  starts: string;
  strand: Strand;
};

type SegmentFetchRequest = {
  assemblyAccession: string;
  ends: number[];
  sequenceName: string;
  starts: number[];
  strand: Strand;
};

const refgetUrl = (request: RefgetRequest): string => {
  const params = new URLSearchParams();
  if (request.start !== undefined) {
    params.set("start", String(request.start));
  }
  if (request.end !== undefined) {
    params.set("end", String(request.end));
  }
  const query = params.toString();
  const path = `/sequence/${encodeURIComponent(request.checksum)}`;
  return query ? `${path}?${query}` : path;
};

const wrapSequence = (sequence: string): string => {
  const lines: string[] = [];
  for (let index = 0; index < sequence.length; index += sequenceLineLength) {
    lines.push(sequence.slice(index, index + sequenceLineLength));
  }
  return lines.join("\n");
};

const FetchPage = (): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <RefgetFetchPanel />
    <SegmentFetchPanel />
  </section>
);

const RefgetFetchPanel = (): ReactElement => {
  const [form, setForm] = useState<FetchForm>({
    checksum: "",
    end: "",
    start: "",
  });
  const [request, setRequest] = useState<RefgetRequest | undefined>();
  const sequenceQuery = useQuery({
    ...refgetSequenceOptions({
      path: { checksum: request?.checksum ?? "" },
      query: {
        end: request?.end,
        start: request?.start,
      },
    }),
    enabled: request !== undefined,
  });

  const onChange = (event: ChangeEvent<HTMLInputElement>): void => {
    setForm((current) => ({ ...current, [event.target.name]: event.target.value }));
  };
  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    setRequest({
      checksum: form.checksum.trim(),
      end: parseOptionalNumber(form.end),
      start: parseOptionalNumber(form.start),
    });
  };

  return (
    <>
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="text-2xl font-semibold">Fetch</h2>
            <p className="mt-2 text-sm text-text-muted">GA4GH refget sequence</p>
          </div>
          {request && sequenceQuery.data !== undefined && (
            <a
              className="inline-flex min-h-10 items-center rounded-md border border-border bg-surface px-3 text-sm font-medium text-text transition hover:bg-surface-muted"
              download={`${request.checksum}.txt`}
              href={refgetUrl(request)}
            >
              Download
            </a>
          )}
        </div>

        <form className="mt-6 grid grid-cols-12 gap-4" onSubmit={onSubmit}>
          <label className="col-span-12 flex flex-col gap-1 md:col-span-6">
            <span className="text-xs font-medium uppercase text-text-subtle">Refget checksum</span>
            <input
              aria-label="Refget checksum"
              className="min-h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="checksum"
              onChange={onChange}
              required
              value={form.checksum}
            />
          </label>
          <label className="col-span-6 flex flex-col gap-1 md:col-span-2">
            <span className="text-xs font-medium uppercase text-text-subtle">Start</span>
            <input
              aria-label="Start"
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              min="0"
              name="start"
              onChange={onChange}
              step="1"
              type="number"
              value={form.start}
            />
          </label>
          <label className="col-span-6 flex flex-col gap-1 md:col-span-2">
            <span className="text-xs font-medium uppercase text-text-subtle">End</span>
            <input
              aria-label="End"
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              min="0"
              name="end"
              onChange={onChange}
              step="1"
              type="number"
              value={form.end}
            />
          </label>
          <div className="col-span-12 flex items-end md:col-span-3">
            <button
              className="min-h-10 rounded-md bg-primary-700 px-4 text-sm font-semibold text-white transition hover:bg-primary-800 disabled:cursor-not-allowed disabled:bg-text-disabled"
              disabled={sequenceQuery.isFetching || form.checksum.trim() === ""}
              type="submit"
            >
              {sequenceQuery.isFetching ? "Fetching" : "Fetch"}
            </button>
          </div>
        </form>
      </div>

      <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
        <div className="border-b border-border-subtle px-6 py-4">
          <h3 className="text-base font-semibold">Sequence</h3>
        </div>
        <FetchResult
          error={sequenceQuery.error}
          isLoading={sequenceQuery.isLoading}
          sequence={sequenceQuery.data}
        />
      </div>
    </>
  );
};

const SegmentFetchPanel = (): ReactElement => {
  const [segmentForm, setSegmentForm] = useState<SegmentFetchForm>({
    assemblyAccession: "",
    ends: "",
    sequenceName: "",
    starts: "",
    strand: "forward",
  });
  const [segmentRequest, setSegmentRequest] = useState<SegmentFetchRequest | undefined>();
  const segmentQuery = useQuery({
    ...sequenceSegmentsOptions({
      path: {
        accession: segmentRequest?.assemblyAccession ?? "",
        sequence_name: segmentRequest?.sequenceName ?? "",
      },
      query: {
        end: segmentRequest?.ends ?? [],
        start: segmentRequest?.starts ?? [],
        strand: segmentRequest?.strand ?? "forward",
      },
    }),
    enabled: segmentRequest !== undefined,
  });

  const onSegmentChange = (
    event: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>,
  ): void => {
    setSegmentForm((current) => ({ ...current, [event.target.name]: event.target.value }));
  };
  const onSegmentSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    setSegmentRequest({
      assemblyAccession: segmentForm.assemblyAccession.trim(),
      ends: parseNumberList(segmentForm.ends),
      sequenceName: segmentForm.sequenceName.trim(),
      starts: parseNumberList(segmentForm.starts),
      strand: segmentForm.strand,
    });
  };

  return (
    <>
      <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="text-xl font-semibold">Segment fetch</h2>
            <p className="mt-2 text-sm text-text-muted">
              Portal sequence endpoint with repeated start/end ranges
            </p>
          </div>
          {segmentRequest && segmentQuery.data !== undefined && (
            <a
              className="inline-flex min-h-10 items-center rounded-md border border-border bg-surface px-3 text-sm font-medium text-text transition hover:bg-surface-muted"
              download={`${segmentRequest.sequenceName}.segments.txt`}
              href={sequenceSegmentsUrl({
                assemblyAccession: segmentRequest.assemblyAccession,
                segments: segmentRequest.starts.map((start, index) => ({
                  end: segmentRequest.ends[index] ?? start,
                  start,
                })),
                sequenceName: segmentRequest.sequenceName,
                strand: segmentRequest.strand,
              })}
            >
              Download
            </a>
          )}
        </div>

        <form className="mt-6 grid grid-cols-12 gap-4" onSubmit={onSegmentSubmit}>
          <label className="col-span-12 flex flex-col gap-1 md:col-span-4">
            <span className="text-xs font-medium uppercase text-text-subtle">
              Assembly accession
            </span>
            <input
              aria-label="Assembly accession"
              className="min-h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="assemblyAccession"
              onChange={onSegmentChange}
              required
              value={segmentForm.assemblyAccession}
            />
          </label>
          <label className="col-span-8 flex flex-col gap-1 md:col-span-3">
            <span className="text-xs font-medium uppercase text-text-subtle">Sequence name</span>
            <input
              aria-label="Sequence name"
              className="min-h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="sequenceName"
              onChange={onSegmentChange}
              required
              value={segmentForm.sequenceName}
            />
          </label>
          <label className="col-span-4 flex flex-col gap-1 md:col-span-2">
            <span className="text-xs font-medium uppercase text-text-subtle">Strand</span>
            <select
              aria-label="Strand"
              className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="strand"
              onChange={onSegmentChange}
              value={segmentForm.strand}
            >
              <option value="forward">forward</option>
              <option value="reverse">reverse</option>
              <option value="unknown">unknown</option>
            </select>
          </label>
          <label className="col-span-12 flex flex-col gap-1 md:col-span-4">
            <span className="text-xs font-medium uppercase text-text-subtle">Starts</span>
            <textarea
              aria-label="Starts"
              className="min-h-20 rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="starts"
              onChange={onSegmentChange}
              required
              value={segmentForm.starts}
            />
          </label>
          <label className="col-span-12 flex flex-col gap-1 md:col-span-4">
            <span className="text-xs font-medium uppercase text-text-subtle">Ends</span>
            <textarea
              aria-label="Ends"
              className="min-h-20 rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
              name="ends"
              onChange={onSegmentChange}
              required
              value={segmentForm.ends}
            />
          </label>
          <div className="col-span-12 flex items-end md:col-span-4">
            <button
              className="min-h-10 rounded-md bg-primary-700 px-4 text-sm font-semibold text-white transition hover:bg-primary-800 disabled:cursor-not-allowed disabled:bg-text-disabled"
              disabled={
                segmentQuery.isFetching ||
                segmentForm.assemblyAccession.trim() === "" ||
                segmentForm.sequenceName.trim() === ""
              }
              type="submit"
            >
              {segmentQuery.isFetching ? "Fetching" : "Fetch segments"}
            </button>
          </div>
        </form>
      </div>

      <div className="col-span-12 overflow-hidden rounded-lg border border-border-subtle bg-surface">
        <div className="border-b border-border-subtle px-6 py-4">
          <h3 className="text-base font-semibold">Segment sequence</h3>
        </div>
        <FetchResult
          error={segmentQuery.error}
          isLoading={segmentQuery.isLoading}
          sequence={segmentQuery.data}
        />
      </div>
    </>
  );
};

const parseOptionalNumber = (value: string): number | undefined => {
  const trimmed = value.trim();
  if (trimmed === "") {
    return undefined;
  }
  return Number(trimmed);
};

const parseNumberList = (value: string): number[] =>
  value
    .split(/[\s,]+/u)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map(Number);

const FetchResult = (props: {
  error: unknown;
  isLoading: boolean;
  sequence: string | undefined;
}): ReactElement => {
  if (props.error) {
    return (
      <div className="p-6">
        <ErrorState detail="The refget sequence request failed." title="Fetch failed" />
      </div>
    );
  }
  if (props.isLoading) {
    return <p className="px-6 py-8 text-sm text-text-muted">Loading sequence.</p>;
  }
  if (props.sequence === undefined) {
    return <p className="px-6 py-8 text-sm text-text-muted">No sequence loaded.</p>;
  }
  return (
    <pre className="max-h-[34rem] overflow-auto bg-surface-muted p-6 font-mono text-[12px] leading-5 text-text">
      {wrapSequence(props.sequence)}
    </pre>
  );
};

export default FetchPage;
