import type { AnnotationEvidence, Gene } from "@/api/client/types.gen";
import { describe, expect, it } from "vitest";
import annotationEntries from "./annotationEntries";

const evidence: AnnotationEvidence = { attributes: {}, source: "manual" };

const gene = (annotations: Gene["annotations"]): Gene => ({
  annotations,
  assembly_accession: "GCA_037833805.1",
  attributes: {},
  feature_type: "gene",
  id: "Mp1g00010",
  region: { end: 100, sequence_name: "chr1", start: 0 },
  sequence_name: "chr1",
  strand: "forward",
});

describe("annotationEntries.buildEntries", () => {
  it("maps each annotation kind to id, name and external href", () => {
    expect.assertions(5);

    const entries = annotationEntries.buildEntries(
      gene([
        { evidence, kind: "go_term", name: "biological_process", term_id: "GO:0008150" },
        { evidence, interpro_id: "IPR000001", kind: "inter_pro", name: "Domain" },
        { accession: "PF00001", evidence, kind: "pfam", name: "7tm_1" },
        { accession: "TIGR00001", evidence, kind: "ncbi_fam", name: "fam" },
        { entry_id: "KOG0001", evidence, kind: "kog", name: "kog" },
      ]),
    );

    expect(entries.go).toStrictEqual([
      {
        href: "https://amigo.geneontology.org/amigo/term/GO:0008150",
        id: "GO:0008150",
        name: "biological_process",
      },
    ]);
    expect(entries.interPro).toStrictEqual([
      {
        href: "https://www.ebi.ac.uk/interpro/entry/InterPro/IPR000001/",
        id: "IPR000001",
        name: "Domain",
      },
    ]);
    expect(entries.pfam[0]?.id).toBe("PF00001");
    expect(entries.ncbiFam[0]?.href).toBe(
      "https://www.ncbi.nlm.nih.gov/genome/annotation_prok/evidence/TIGR00001/",
    );
    // KOG has no first-party page, so href is intentionally empty.
    expect(entries.kog).toStrictEqual([{ href: "", id: "KOG0001", name: "kog" }]);
  });

  it("falls back to an empty name when the annotation name is absent", () => {
    expect.assertions(1);

    const entries = annotationEntries.buildEntries(
      gene([{ evidence, kind: "go_term", term_id: "GO:0008150" }]),
    );

    expect(entries.go[0]?.name).toBe("");
  });
});
