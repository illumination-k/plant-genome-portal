import type { Exon } from "@/api/client/types.gen";
import { describe, expect, it } from "vitest";
import geneRecordUtils from "./geneRecordUtils";

const exon = (transcriptId: string, start: number, end: number): Exon => ({
  region: { end, sequence_name: "chr1", start },
  sequence_name: "chr1",
  strand: "forward",
  transcript_id: transcriptId,
});

describe("geneRecordUtils.countExonsByTranscript", () => {
  it("counts exons grouped by transcript id", () => {
    expect.assertions(3);

    const counts = geneRecordUtils.countExonsByTranscript([
      exon("t1", 0, 10),
      exon("t1", 20, 30),
      exon("t2", 0, 5),
    ]);

    expect(counts.get("t1")).toBe(2);
    expect(counts.get("t2")).toBe(1);
    expect(counts.get("missing")).toBeUndefined();
  });

  it("returns an empty map for no exons", () => {
    expect.assertions(1);

    expect(geneRecordUtils.countExonsByTranscript([]).size).toBe(0);
  });
});

describe("geneRecordUtils.errorMessage", () => {
  it("unwraps Error instances and otherwise uses a fallback", () => {
    expect.assertions(2);

    expect(geneRecordUtils.errorMessage(new Error("kaboom"))).toBe("kaboom");
    expect(geneRecordUtils.errorMessage(42)).toBe("The API request failed.");
  });
});

describe("geneRecordUtils.exonKey", () => {
  it("builds a stable key from transcript id and region bounds", () => {
    expect.assertions(1);

    expect(geneRecordUtils.exonKey(exon("t1", 100, 250))).toBe("t1-100-250");
  });
});
