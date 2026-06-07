import type { Cds, Exon, GeneRecord, Transcript } from "@/api/client/types.gen";
import { describe, expect, it } from "vitest";
import geneStructure from "./geneStructure";

const transcript = (id: string, start: number, end: number): Transcript => ({
  annotations: [],
  attributes: {},
  feature_type: "mRNA",
  gene_id: "Mp1g00010",
  id,
  region: { end, sequence_name: "chr1", start },
  sequence_name: "chr1",
  strand: "forward",
});

const exon = (transcriptId: string, start: number, end: number): Exon => ({
  region: { end, sequence_name: "chr1", start },
  sequence_name: "chr1",
  strand: "forward",
  transcript_id: transcriptId,
});

const cds = (transcriptId: string, start: number, end: number): Cds => ({
  phase: 0,
  region: { end, sequence_name: "chr1", start },
  sequence_name: "chr1",
  strand: "forward",
  transcript_id: transcriptId,
});

describe("geneStructure.makeScale", () => {
  it("maps the region start to the left padding and the end to the usable width", () => {
    expect.assertions(3);

    const scale = geneStructure.makeScale(0, 1000);

    expect(scale(0)).toBeCloseTo(16);
    expect(scale(1000)).toBeCloseTo(984);
    expect(scale(500)).toBeCloseTo(500);
  });

  it("clamps a zero-width span to avoid dividing by zero", () => {
    expect.assertions(1);

    const scale = geneStructure.makeScale(5, 5);

    expect(scale(5)).toBeCloseTo(16);
  });
});

describe("geneStructure.totalSvgHeight", () => {
  it("reserves at least one row plus the axis", () => {
    expect.assertions(2);

    expect(geneStructure.totalSvgHeight(0)).toBe(84);
    expect(geneStructure.totalSvgHeight(3)).toBe(196);
  });
});

describe("geneStructure.computeAxisY", () => {
  it("places the axis below the rows", () => {
    expect.assertions(1);

    expect(geneStructure.computeAxisY(2)).toBeCloseTo(112 + 28 / 3);
  });
});

describe("geneStructure.isEmpty", () => {
  it("is true only for a zero count", () => {
    expect.assertions(2);

    expect(geneStructure.isEmpty(0)).toBe(true);
    expect(geneStructure.isEmpty(1)).toBe(false);
  });
});

describe("geneStructure.groupByTranscript", () => {
  it("groups and sorts exons and cdss per transcript by start", () => {
    expect.assertions(4);

    const record: GeneRecord = {
      cdss: [cds("t1", 50, 100)],
      exons: [exon("t1", 400, 500), exon("t1", 0, 100), exon("t2", 0, 50)],
      gene: {
        annotations: [],
        assembly_accession: "GCA_037833805.1",
        attributes: {},
        feature_type: "gene",
        id: "Mp1g00010",
        region: { end: 500, sequence_name: "chr1", start: 0 },
        sequence_name: "chr1",
        strand: "forward",
      },
      transcripts: [transcript("t1", 0, 500), transcript("t2", 0, 50)],
    };

    const groups = geneStructure.groupByTranscript(record);

    expect(groups).toHaveLength(2);
    // Exons for t1 must be sorted ascending by start.
    expect(groups[0]?.exons.map((entry) => entry.region.start)).toStrictEqual([0, 400]);
    expect(groups[0]?.cdss).toHaveLength(1);
    expect(groups[1]?.exons.map((entry) => entry.transcript_id)).toStrictEqual(["t2"]);
  });
});

describe("geneStructure.axisTokens", () => {
  it("emits start/middle/end ticks with 1-based labels", () => {
    expect.assertions(4);

    const scale = geneStructure.makeScale(0, 1000);
    const axis = geneStructure.axisTokens({ end: 1000, posY: 200, scale, start: 0 });

    expect(axis.ticks).toHaveLength(3);
    expect(axis.ticks.map((tick) => tick.anchor)).toStrictEqual(["start", "middle", "end"]);
    expect(axis.ticks.map((tick) => tick.label)).toStrictEqual(["1", "501", "1,000"]);
    expect(axis.posY).toBe(200);
  });
});

describe("geneStructure.trackTokens", () => {
  it("builds exon and cds boxes with 1-based titles and centred geometry", () => {
    expect.assertions(6);

    const scale = geneStructure.makeScale(0, 1000);
    const group = {
      cdss: [cds("t1", 50, 100)],
      exons: [exon("t1", 0, 100), exon("t1", 400, 500)],
      transcript: transcript("t1", 0, 1000),
    };

    const tokens = geneStructure.trackTokens(group, scale, 0);

    expect(tokens.exonBoxes).toHaveLength(2);
    // Renders 0-based [0,100) as the 1-based closed label 1-100 spanning 100 bp.
    expect(tokens.exonBoxes[0]?.title).toBe("Exon 1-100 (100 bp)");
    expect(tokens.exonBoxes[0]?.height).toBe(10);
    expect(tokens.cdsBoxes[0]?.title).toBe("CDS 51-100 (50 bp, phase 0)");
    // CDS boxes are taller than exon boxes.
    expect(tokens.cdsBoxes[0]?.height).toBe(18);
    expect(tokens.label).toStrictEqual({
      id: "t1",
      posX: scale(0),
      posY: 12,
      title: "t1 · mRNA",
    });
  });

  it("draws strand chevrons inside wide introns", () => {
    expect.assertions(2);

    const scale = geneStructure.makeScale(0, 1000);
    const group = {
      cdss: [],
      exons: [exon("t1", 0, 100), exon("t1", 400, 500)],
      transcript: transcript("t1", 0, 1000),
    };

    const tokens = geneStructure.trackTokens(group, scale, 0);

    // Two wide gaps (100-400 and 500-1000) both clear the chevron threshold.
    expect(tokens.chevrons.length).toBeGreaterThan(0);
    expect(tokens.chevrons.every((chevron) => chevron.pathD.startsWith("M "))).toBe(true);
  });

  it("omits chevrons when a single exon spans the whole transcript", () => {
    expect.assertions(1);

    const scale = geneStructure.makeScale(0, 1000);
    const group = {
      cdss: [],
      exons: [exon("t1", 0, 1000)],
      transcript: transcript("t1", 0, 1000),
    };

    const tokens = geneStructure.trackTokens(group, scale, 0);

    expect(tokens.chevrons).toHaveLength(0);
  });
});
