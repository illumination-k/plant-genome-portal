import {
  assemblySequencesOptions,
  refgetSequenceOptions,
} from "@/api/client/@tanstack/react-query.gen";
import { useQuery } from "@tanstack/react-query";

type GeneRefgetSequence = {
  checksum: string;
  hasRequestError: boolean;
  hasSequenceMetadata: boolean;
  isLoading: boolean;
  isMetadataLoading: boolean;
  sequence: string;
};

type GeneRefgetSequenceRequest = {
  assemblyAccession: string;
  chr: string;
  end: number;
  start: number;
};

const useGeneRefgetSequence = (request: GeneRefgetSequenceRequest): GeneRefgetSequence => {
  const sequencesQuery = useQuery(
    assemblySequencesOptions({ path: { accession: request.assemblyAccession } }),
  );
  const sequenceMetadata = sequencesQuery.data?.find((sequence) => sequence.name === request.chr);
  const checksum = sequenceMetadata?.refget_checksum ?? "";
  const sequenceQuery = useQuery({
    ...refgetSequenceOptions({
      path: { checksum },
      query: { end: request.end, start: request.start },
    }),
    enabled: checksum !== "",
  });
  return {
    checksum,
    hasRequestError: Boolean(sequencesQuery.error || sequenceQuery.error),
    hasSequenceMetadata: Boolean(sequenceMetadata),
    isLoading: sequencesQuery.isLoading || sequenceQuery.isLoading,
    isMetadataLoading: sequencesQuery.isLoading,
    sequence: sequenceQuery.data ?? "",
  };
};

export default useGeneRefgetSequence;
