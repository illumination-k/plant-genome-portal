import type { Strand } from "@/api/client";

type SequenceSegmentsUrlRequest = {
  assemblyAccession: string;
  format?: "fasta" | "plain";
  segments: Array<{ end: number; start: number }>;
  sequenceName: string;
  strand?: Strand;
};

const sequenceSegmentsUrl = (request: SequenceSegmentsUrlRequest): string => {
  const params = new URLSearchParams();
  for (const segment of request.segments) {
    params.append("start", String(segment.start));
    params.append("end", String(segment.end));
  }
  if (request.strand !== undefined) {
    params.set("strand", request.strand);
  }
  if (request.format !== undefined) {
    params.set("format", request.format);
  }
  const path = [
    "/v2/genome/accession",
    encodeURIComponent(request.assemblyAccession),
    "sequence",
    encodeURIComponent(request.sequenceName),
  ].join("/");
  return `${path}?${params.toString()}`;
};

export default sequenceSegmentsUrl;
