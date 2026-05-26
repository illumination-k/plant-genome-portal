import type { Gene } from "@/api/client/types.gen";
import functionalAnnotation from "@/shared/lib/functionalAnnotation";

type Entry = {
  href: string;
  id: string;
  name: string;
};

type GroupedEntries = {
  go: Entry[];
  interPro: Entry[];
  kog: Entry[];
  ncbiFam: Entry[];
  pfam: Entry[];
};

const safeName = (name: string | null | undefined): string => name ?? "";

const buildEntries = (gene: Gene): GroupedEntries => {
  const grouped = functionalAnnotation.group(gene.annotations);
  return {
    go: grouped.go.map((annotation) => ({
      href: functionalAnnotation.externalLink(annotation),
      id: annotation.term_id,
      name: safeName(annotation.name),
    })),
    interPro: grouped.interPro.map((annotation) => ({
      href: functionalAnnotation.externalLink(annotation),
      id: annotation.interpro_id,
      name: safeName(annotation.name),
    })),
    kog: grouped.kog.map((annotation) => ({
      href: "",
      id: annotation.entry_id,
      name: safeName(annotation.name),
    })),
    ncbiFam: grouped.ncbiFam.map((annotation) => ({
      href: functionalAnnotation.externalLink(annotation),
      id: annotation.accession,
      name: safeName(annotation.name),
    })),
    pfam: grouped.pfam.map((annotation) => ({
      href: functionalAnnotation.externalLink(annotation),
      id: annotation.accession,
      name: safeName(annotation.name),
    })),
  };
};

const annotationEntries = {
  buildEntries,
};

export default annotationEntries;
