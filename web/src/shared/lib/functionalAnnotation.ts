import type { FunctionalAnnotation } from "@/api/client/types.gen";

type Grouped = {
  go: Array<Extract<FunctionalAnnotation, { kind: "go_term" }>>;
  interPro: Array<Extract<FunctionalAnnotation, { kind: "inter_pro" }>>;
  kegg: Array<Extract<FunctionalAnnotation, { kind: "kegg" }>>;
  kog: Array<Extract<FunctionalAnnotation, { kind: "kog" }>>;
  ncbiFam: Array<Extract<FunctionalAnnotation, { kind: "ncbi_fam" }>>;
  pfam: Array<Extract<FunctionalAnnotation, { kind: "pfam" }>>;
};

const empty = (): Grouped => ({
  go: [],
  interPro: [],
  kegg: [],
  kog: [],
  ncbiFam: [],
  pfam: [],
});

const group = (annotations: FunctionalAnnotation[]): Grouped => {
  const out = empty();
  const byKind = {
    go_term: out.go,
    inter_pro: out.interPro,
    kegg: out.kegg,
    kog: out.kog,
    ncbi_fam: out.ncbiFam,
    pfam: out.pfam,
  };
  for (const annotation of annotations) {
    (byKind[annotation.kind] as FunctionalAnnotation[]).push(annotation);
  }
  return out;
};

const externalLink = (annotation: FunctionalAnnotation): string => {
  switch (annotation.kind) {
    case "go_term": {
      return `https://amigo.geneontology.org/amigo/term/${annotation.term_id}`;
    }
    case "inter_pro": {
      return `https://www.ebi.ac.uk/interpro/entry/InterPro/${annotation.interpro_id}/`;
    }
    case "pfam": {
      return `https://www.ebi.ac.uk/interpro/entry/pfam/${annotation.accession}/`;
    }
    case "kegg": {
      return `https://www.kegg.jp/entry/${annotation.entry_id}`;
    }
    case "ncbi_fam": {
      return `https://www.ncbi.nlm.nih.gov/genome/annotation_prok/evidence/${annotation.accession}/`;
    }
    default: {
      // KOG has no first-party page; fall through.
      return "";
    }
  }
};

const functionalAnnotation = {
  externalLink,
  group,
};

export default functionalAnnotation;
