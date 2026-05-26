/* oxlint-disable no-magic-numbers, jsx-max-depth */
import type {
  EnrichmentAnnotationKind,
  EnrichmentTerm,
  EnrichmentTermResult,
} from "@/api/client/types.gen";
import GeneIdLink from "@/shared/bio/GeneIdLink";
import type { ReactElement, ReactNode } from "react";

const annotationKindLabels: Record<EnrichmentAnnotationKind, string> = {
  go_term: "GO",
  inter_pro: "InterPro",
  kegg: "KEGG",
  kog: "KOG",
  ncbi_fam: "NCBIfam",
  pfam: "Pfam",
};

const formatScore = (value: number): string => {
  if (value === 0) {
    return "0";
  }
  if (value < 0.001) {
    return value.toExponential(2);
  }
  return value.toFixed(4);
};

const formatFold = (value: number | null | undefined): string => {
  if (value === undefined || value === null) {
    return "n/a";
  }
  return value.toFixed(2);
};

const termHref = (term: EnrichmentTerm): string => {
  switch (term.kind) {
    case "go_term": {
      return `https://amigo.geneontology.org/amigo/term/${term.id}`;
    }
    case "inter_pro": {
      return `https://www.ebi.ac.uk/interpro/entry/InterPro/${term.id}/`;
    }
    case "pfam": {
      return `https://www.ebi.ac.uk/interpro/entry/pfam/${term.id}/`;
    }
    case "kegg": {
      return `https://www.kegg.jp/entry/${term.id}`;
    }
    case "ncbi_fam": {
      return `https://www.ncbi.nlm.nih.gov/genome/annotation_prok/evidence/${term.id}/`;
    }
    default: {
      return "";
    }
  }
};

const renderTermId = (href: string, id: ReactNode): ReactNode => {
  if (!href) {
    return id;
  }
  return (
    <a href={href} rel="noreferrer" target="_blank">
      {id}
    </a>
  );
};

const TermCell = (props: { term: EnrichmentTerm }): ReactElement => {
  const href = termHref(props.term);
  const id = <span className="font-mono text-[12px] font-semibold text-text">{props.term.id}</span>;
  const linkedId = renderTermId(href, id);
  return (
    <div className="min-w-48">
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded border border-border-subtle bg-surface-muted px-1.5 py-0.5 text-[11px] font-semibold text-text-muted">
          {annotationKindLabels[props.term.kind]}
        </span>
        {linkedId}
      </div>
      <p className="mt-1 max-w-96 text-[12px] leading-5 text-text-muted">
        {props.term.name ?? props.term.namespace ?? "No term name"}
      </p>
    </div>
  );
};

const GeneHitLinks = (props: { geneIds: string[] }): ReactElement => {
  const visible = props.geneIds.slice(0, 6);
  const overflow = props.geneIds.length - visible.length;
  return (
    <div className="flex max-w-72 flex-wrap gap-x-2 gap-y-1">
      {visible.map((geneId) => (
        <GeneIdLink geneId={geneId} key={geneId} />
      ))}
      {overflow > 0 && <span className="text-[12px] text-text-muted">+{overflow}</span>}
    </div>
  );
};

const EnrichmentResultRow = (props: { result: EnrichmentTermResult }): ReactElement => (
  <tr className="border-b border-border-subtle last:border-b-0">
    <td className="px-3 py-3 align-top">
      <TermCell term={props.result.term} />
    </td>
    <td className="px-3 py-3 text-right align-top font-mono text-[12px]">
      {props.result.studyHits}/{props.result.studySize}
    </td>
    <td className="px-3 py-3 text-right align-top font-mono text-[12px]">
      {props.result.populationHits}/{props.result.populationSize}
    </td>
    <td className="px-3 py-3 text-right align-top font-mono text-[12px]">
      {formatFold(props.result.foldEnrichment)}
    </td>
    <td className="px-3 py-3 text-right align-top font-mono text-[12px]">
      {formatScore(props.result.pValue)}
    </td>
    <td className="px-3 py-3 text-right align-top font-mono text-[12px]">
      {formatScore(props.result.qValue)}
    </td>
    <td className="px-3 py-3 align-top">
      <GeneHitLinks geneIds={props.result.studyGeneIds} />
    </td>
  </tr>
);

const EnrichmentResultsTable = (props: { results: EnrichmentTermResult[] }): ReactElement => (
  <div className="overflow-x-auto rounded-md border border-border-subtle">
    <table className="w-full min-w-[920px] border-collapse text-left text-sm">
      <thead className="bg-surface-muted text-[11px] uppercase tracking-wide text-text-subtle">
        <tr>
          <th className="px-3 py-2 font-semibold">Term</th>
          <th className="px-3 py-2 text-right font-semibold">Study</th>
          <th className="px-3 py-2 text-right font-semibold">Population</th>
          <th className="px-3 py-2 text-right font-semibold">Fold</th>
          <th className="px-3 py-2 text-right font-semibold">p-value</th>
          <th className="px-3 py-2 text-right font-semibold">q-value</th>
          <th className="px-3 py-2 font-semibold">Hits</th>
        </tr>
      </thead>
      <tbody>
        {props.results.map((result) => (
          <EnrichmentResultRow key={`${result.term.kind}:${result.term.id}`} result={result} />
        ))}
      </tbody>
    </table>
  </div>
);

export default EnrichmentResultsTable;
