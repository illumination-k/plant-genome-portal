import type { GeneRecord } from "@/api/client";
import type { ReactElement } from "react";
import GeneSequenceBody from "@/features/genes/components/GeneSequenceBody";
import GeneSequenceCardHeader from "@/features/genes/components/GeneSequenceCardHeader";
import GeneSequenceDownloads from "@/features/genes/components/GeneSequenceDownloads";
import GeneSequenceErrors from "@/features/genes/components/GeneSequenceErrors";
import GeneSequenceMetadata from "@/features/genes/components/GeneSequenceMetadata";
import useGeneRefgetSequence from "@/features/genes/components/useGeneRefgetSequence";

const oneBasedOffset = 1;

const refgetUrl = (checksum: string, start: number, end: number): string => {
  const params = new URLSearchParams({
    end: String(end),
    start: String(start),
  });
  return `/sequence/${encodeURIComponent(checksum)}?${params.toString()}`;
};

const optionalRefgetUrl = (checksum: string, start: number, end: number): string | undefined => {
  if (checksum === "") {
    return undefined;
  }
  return refgetUrl(checksum, start, end);
};

const GeneSequenceCard = (props: {
  assemblyAccession: string;
  chr: string;
  end: number;
  geneRecord: GeneRecord;
  geneId: string;
  length: number;
  start: number;
}): ReactElement => {
  const start0 = props.start - oneBasedOffset;
  const refget = useGeneRefgetSequence({
    assemblyAccession: props.assemblyAccession,
    chr: props.chr,
    end: props.end,
    start: start0,
  });
  const endpointUrl = optionalRefgetUrl(refget.checksum, start0, props.end);

  return (
    <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
      <GeneSequenceCardHeader downloadUrl={endpointUrl} geneId={props.geneId} />
      <GeneSequenceMetadata
        chr={props.chr}
        endpointUrl={endpointUrl}
        end={props.end}
        length={props.length}
        start={props.start}
      />
      <GeneSequenceBody isLoading={refget.isLoading} sequence={refget.sequence} />
      <GeneSequenceErrors
        hasRequestError={refget.hasRequestError}
        hasSequenceMetadata={refget.hasSequenceMetadata}
        isMetadataLoading={refget.isMetadataLoading}
      />
      <GeneSequenceDownloads geneRecord={props.geneRecord} />
    </div>
  );
};

export default GeneSequenceCard;
