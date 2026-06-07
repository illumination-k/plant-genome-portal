import { describe, expect, it } from "vitest";
import geneFormat from "./geneFormat";

describe("geneFormat.formatPosition", () => {
  it("formats integers with en-US thousands separators", () => {
    expect.assertions(2);

    expect(geneFormat.formatPosition(1_234_567)).toBe("1,234,567");
    expect(geneFormat.formatPosition(0)).toBe("0");
  });
});

describe("geneFormat.formatLocation", () => {
  it("converts a 0-based half-open region into a 1-based closed label", () => {
    expect.assertions(1);

    const label = geneFormat.formatLocation("chr1", {
      end: 100,
      sequence_name: "chr1",
      start: 0,
    });

    expect(label).toBe("chr1:1-100");
  });
});

describe("geneFormat.formatRegion", () => {
  it("renders the 1-based start and inclusive end with separators", () => {
    expect.assertions(1);

    const region = geneFormat.formatRegion({
      annotations: [],
      assembly_accession: "GCA_037833805.1",
      attributes: {},
      feature_type: "gene",
      id: "Mp1g00010",
      region: { end: 12_000, sequence_name: "chr1", start: 999 },
      sequence_name: "chr1",
      strand: "forward",
    });

    expect(region).toBe("1,000-12,000");
  });
});

describe("geneFormat.formatStrand", () => {
  it("maps forward to +", () => {
    expect.assertions(1);

    expect(geneFormat.formatStrand("forward")).toBe("+");
  });

  it("maps reverse to -", () => {
    expect.assertions(1);

    expect(geneFormat.formatStrand("reverse")).toBe("-");
  });

  it("maps unknown to a dot", () => {
    expect.assertions(1);

    expect(geneFormat.formatStrand("unknown")).toBe(".");
  });
});

describe("geneFormat.getErrorMessage", () => {
  it("returns the message of an Error instance", () => {
    expect.assertions(1);

    expect(geneFormat.getErrorMessage(new Error("boom"))).toBe("boom");
  });

  it("falls back to a generic message for non-Error values", () => {
    expect.assertions(2);

    expect(geneFormat.getErrorMessage("nope")).toBe("The API request failed.");
    expect(geneFormat.getErrorMessage(404)).toBe("The API request failed.");
  });
});
