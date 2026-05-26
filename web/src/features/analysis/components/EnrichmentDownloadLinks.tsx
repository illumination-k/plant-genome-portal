import type { EnrichmentAnalysisResponse2, EnrichmentTermResult } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useMemo } from "react";

const tsvHeader = [
  "annotation_kind",
  "term_id",
  "term_name",
  "namespace",
  "study_hits",
  "study_size",
  "population_hits",
  "population_size",
  "fold_enrichment",
  "p_value",
  "q_value",
  "study_gene_ids",
];

const jsonIndentSpaces = 2;

const escapeTsvCell = (value: string): string => value.replaceAll("\t", " ").replaceAll("\n", " ");

const resultToTsvRow = (result: EnrichmentTermResult): string =>
  [
    result.term.kind,
    result.term.id,
    result.term.name ?? "",
    result.term.namespace ?? "",
    String(result.studyHits),
    String(result.studySize),
    String(result.populationHits),
    String(result.populationSize),
    result.foldEnrichment?.toString() ?? "",
    result.pValue.toString(),
    result.qValue.toString(),
    result.studyGeneIds.join(","),
  ]
    .map((value) => escapeTsvCell(value))
    .join("\t");

const enrichmentResultTsv = (result: EnrichmentAnalysisResponse2): string =>
  `${[
    tsvHeader.join("\t"),
    ...result.results.map((enrichmentResult) => resultToTsvRow(enrichmentResult)),
  ].join("\n")}\n`;

const downloadHref = (content: string, contentType: string): string =>
  `data:${contentType};charset=utf-8,${encodeURIComponent(content)}`;

const downloadBasename = (result: EnrichmentAnalysisResponse2): string =>
  `${result.assemblyAccession}.enrichment`;

const EnrichmentDownloadLinks = (props: { result: EnrichmentAnalysisResponse2 }): ReactElement => {
  const downloadLinks = useMemo(() => {
    const basename = downloadBasename(props.result);
    return {
      json: {
        filename: `${basename}.json`,
        href: downloadHref(
          JSON.stringify(props.result, undefined, jsonIndentSpaces),
          "application/json",
        ),
      },
      tsv: {
        filename: `${basename}.tsv`,
        href: downloadHref(enrichmentResultTsv(props.result), "text/tab-separated-values"),
      },
    };
  }, [props.result]);

  return (
    <>
      <a
        className="rounded-md border border-border-subtle bg-surface px-2 py-1 font-semibold text-primary-800 hover:bg-surface-muted hover:text-primary-900"
        download={downloadLinks.tsv.filename}
        href={downloadLinks.tsv.href}
      >
        Download TSV
      </a>
      <a
        className="rounded-md border border-border-subtle bg-surface px-2 py-1 font-semibold text-primary-800 hover:bg-surface-muted hover:text-primary-900"
        download={downloadLinks.json.filename}
        href={downloadLinks.json.href}
      >
        Download JSON
      </a>
    </>
  );
};

export default EnrichmentDownloadLinks;
