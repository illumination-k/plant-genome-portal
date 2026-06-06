import { describe, expect, it } from "vitest";
import sequenceSegmentsUrl from "./sequenceSegmentsUrl";

describe("sequenceSegmentsUrl", () => {
  it("emits one start/end pair per segment in order", () => {
    expect.assertions(1);

    const url = sequenceSegmentsUrl({
      assemblyAccession: "GCA_037833805.1",
      segments: [
        { end: 200, start: 100 },
        { end: 400, start: 300 },
      ],
      sequenceName: "chr1",
    });

    expect(url).toBe(
      "/v2/genome/accession/GCA_037833805.1/sequence/chr1?start=100&end=200&start=300&end=400",
    );
  });

  it("appends strand and format only when provided", () => {
    expect.assertions(1);

    const url = sequenceSegmentsUrl({
      assemblyAccession: "GCA_037833805.1",
      format: "fasta",
      segments: [{ end: 50, start: 0 }],
      sequenceName: "chr1",
      strand: "reverse",
    });

    expect(url).toBe(
      "/v2/genome/accession/GCA_037833805.1/sequence/chr1?start=0&end=50&strand=reverse&format=fasta",
    );
  });

  it("URL-encodes assembly and sequence path segments", () => {
    expect.assertions(1);

    const url = sequenceSegmentsUrl({
      assemblyAccession: "local/draft 1",
      segments: [{ end: 10, start: 0 }],
      sequenceName: "scaffold 7",
    });

    expect(url).toContain("/v2/genome/accession/local%2Fdraft%201/sequence/scaffold%207?");
  });
});
