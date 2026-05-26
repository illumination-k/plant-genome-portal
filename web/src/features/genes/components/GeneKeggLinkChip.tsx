import type { ReactElement } from "react";
import FunctionalAnnotationChip from "@/shared/bio/FunctionalAnnotationChip";

const keggEntryHref = (id: string): string => `https://www.kegg.jp/entry/${id}`;
const safeName = (name: string | null | undefined): string => name ?? "";

const GeneKeggLinkChip = (props: {
  href?: string;
  id: string;
  name: string | null | undefined;
}): ReactElement => (
  <FunctionalAnnotationChip
    href={props.href ?? keggEntryHref(props.id)}
    id={props.id}
    name={safeName(props.name)}
  />
);

export default GeneKeggLinkChip;
