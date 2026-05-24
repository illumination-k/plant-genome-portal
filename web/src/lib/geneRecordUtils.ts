import type { Exon } from "@/api/client/types.gen";

const emptyCount = 0;
const oneBasedOffset = 1;

const countExonsByTranscript = (exons: Exon[]): Map<string, number> => {
  const counts = new Map<string, number>();

  for (const exon of exons) {
    counts.set(exon.transcript_id, (counts.get(exon.transcript_id) ?? emptyCount) + oneBasedOffset);
  }

  return counts;
};

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }

  return "The API request failed.";
};

const exonKey = (exon: Exon): string =>
  `${exon.transcript_id}-${exon.region.start}-${exon.region.end}`;

const geneRecordUtils = {
  countExonsByTranscript,
  errorMessage,
  exonKey,
};

export default geneRecordUtils;
