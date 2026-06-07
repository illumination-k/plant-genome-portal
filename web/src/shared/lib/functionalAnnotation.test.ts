import type { AnnotationEvidence, FunctionalAnnotation } from "@/api/client/types.gen";
import { describe, expect, it } from "vitest";
import functionalAnnotation from "./functionalAnnotation";

const evidence: AnnotationEvidence = { attributes: {}, source: "manual" };

const go: FunctionalAnnotation = { evidence, kind: "go_term", term_id: "GO:0008150" };
const interPro: FunctionalAnnotation = { evidence, interpro_id: "IPR000001", kind: "inter_pro" };
const pfam: FunctionalAnnotation = { accession: "PF00001", evidence, kind: "pfam" };
const ncbiFam: FunctionalAnnotation = { accession: "TIGR00001", evidence, kind: "ncbi_fam" };
const kog: FunctionalAnnotation = { entry_id: "KOG0001", evidence, kind: "kog" };
const kegg: FunctionalAnnotation = {
  entry_id: "K00001",
  entry_kind: "orthology",
  evidence,
  kind: "kegg",
};

describe("functionalAnnotation.group", () => {
  it("partitions annotations into their kind buckets", () => {
    expect.assertions(6);

    const grouped = functionalAnnotation.group([go, interPro, pfam, ncbiFam, kog, kegg]);

    expect(grouped.go).toStrictEqual([go]);
    expect(grouped.interPro).toStrictEqual([interPro]);
    expect(grouped.pfam).toStrictEqual([pfam]);
    expect(grouped.ncbiFam).toStrictEqual([ncbiFam]);
    expect(grouped.kog).toStrictEqual([kog]);
    expect(grouped.kegg).toStrictEqual([kegg]);
  });

  it("returns empty buckets for no annotations", () => {
    expect.assertions(1);

    const grouped = functionalAnnotation.group([]);

    expect(grouped).toStrictEqual({
      go: [],
      interPro: [],
      kegg: [],
      kog: [],
      ncbiFam: [],
      pfam: [],
    });
  });
});

describe("functionalAnnotation.externalLink", () => {
  it("builds an AmiGO link for GO terms", () => {
    expect.assertions(1);

    expect(functionalAnnotation.externalLink(go)).toBe(
      "https://amigo.geneontology.org/amigo/term/GO:0008150",
    );
  });

  it("builds InterPro links for InterPro and Pfam", () => {
    expect.assertions(2);

    expect(functionalAnnotation.externalLink(interPro)).toBe(
      "https://www.ebi.ac.uk/interpro/entry/InterPro/IPR000001/",
    );
    expect(functionalAnnotation.externalLink(pfam)).toBe(
      "https://www.ebi.ac.uk/interpro/entry/pfam/PF00001/",
    );
  });

  it("builds a KEGG link from the entry id", () => {
    expect.assertions(1);

    expect(functionalAnnotation.externalLink(kegg)).toBe("https://www.kegg.jp/entry/K00001");
  });

  it("builds an NCBIfam evidence link", () => {
    expect.assertions(1);

    expect(functionalAnnotation.externalLink(ncbiFam)).toBe(
      "https://www.ncbi.nlm.nih.gov/genome/annotation_prok/evidence/TIGR00001/",
    );
  });

  it("returns an empty string for KOG which has no first-party page", () => {
    expect.assertions(1);

    expect(functionalAnnotation.externalLink(kog)).toBe("");
  });
});
