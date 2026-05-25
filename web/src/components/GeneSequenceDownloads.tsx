import type { GeneRecord, Strand, Transcript } from "@/api/client";
import type { ReactElement } from "react";
import sequenceSegmentsUrl from "@/lib/sequenceSegmentsUrl";

const noSegments = 0;

type DownloadLink = {
  filename: string;
  label: string;
  url: string;
};

type SequenceSegment = {
  end: number;
  start: number;
};

const sortSegments = (segments: SequenceSegment[]): SequenceSegment[] =>
  segments.toSorted(
    (left, right) => left.start - right.start || left.end - right.end,
  );

const transcriptLabel = (transcript: Transcript): string => transcript.id;

const transcriptSegments = (
  geneRecord: GeneRecord,
  transcript: Transcript,
  kind: "exons" | "cds",
): SequenceSegment[] => {
  let features = geneRecord.cdss;
  if (kind === "exons") {
    features = geneRecord.exons;
  }
  return sortSegments(
    features
      .filter((feature) => feature.transcript_id === transcript.id)
      .map((feature) => ({
        end: feature.region.end,
        start: feature.region.start,
      })),
  );
};

const makeUrl = (
  geneRecord: GeneRecord,
  segments: SequenceSegment[],
  strand?: Strand,
): string =>
  sequenceSegmentsUrl({
    assemblyAccession: geneRecord.gene.assembly_accession,
    format: "fasta",
    segments,
    sequenceName: geneRecord.gene.sequence_name,
    strand,
  });

const appendTranscriptLinks = (
  links: DownloadLink[],
  geneRecord: GeneRecord,
  transcript: Transcript,
): void => {
  const { gene } = geneRecord;
  const exonSegments = transcriptSegments(geneRecord, transcript, "exons");
  if (exonSegments.length > noSegments) {
    links.push({
      filename: `${gene.id}.${transcript.id}.exons.fa`,
      label: `${transcriptLabel(transcript)} exons`,
      url: makeUrl(geneRecord, exonSegments, gene.strand),
    });
  }

  const cdsSegments = transcriptSegments(geneRecord, transcript, "cds");
  if (cdsSegments.length > noSegments) {
    links.push({
      filename: `${gene.id}.${transcript.id}.cds.fa`,
      label: `${transcriptLabel(transcript)} CDS`,
      url: makeUrl(geneRecord, cdsSegments, gene.strand),
    });
  }
};

const buildDownloadLinks = (geneRecord: GeneRecord): DownloadLink[] => {
  const { gene } = geneRecord;
  const links: DownloadLink[] = [
    {
      filename: `${gene.id}.genomic.fa`,
      label: "Gene span",
      url: makeUrl(
        geneRecord,
        [
          {
            end: gene.region.end,
            start: gene.region.start,
          },
        ],
      ),
    },
  ];

  for (const transcript of geneRecord.transcripts) {
    appendTranscriptLinks(links, geneRecord, transcript);
  }

  return links;
};

const GeneSequenceDownloads = (props: { geneRecord: GeneRecord }): ReactElement => {
  const links = buildDownloadLinks(props.geneRecord);
  return (
    <section className="mt-5 border-t border-border-subtle pt-5">
      <h4 className="text-sm font-semibold text-text">Downloads</h4>
      <div className="mt-3 flex flex-wrap gap-2">
        {links.map((link) => (
          <a
            className="inline-flex min-h-9 items-center rounded-md border border-border bg-surface px-3 text-sm font-medium text-text transition hover:bg-surface-muted"
            download={link.filename}
            href={link.url}
            key={link.filename}
          >
            {link.label}
          </a>
        ))}
      </div>
    </section>
  );
};

export default GeneSequenceDownloads;
