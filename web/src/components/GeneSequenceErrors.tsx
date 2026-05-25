import type { ReactElement } from "react";
import ErrorState from "@/ui/ErrorState";

const GeneSequenceErrors = (props: {
  hasRequestError: boolean;
  hasSequenceMetadata: boolean;
  isMetadataLoading: boolean;
}): ReactElement | false => {
  if (!props.hasSequenceMetadata && !props.isMetadataLoading) {
    return (
      <div className="mt-5">
        <ErrorState detail="No refget checksum was found for this sequence." title="Fetch failed" />
      </div>
    );
  }
  if (props.hasRequestError) {
    return (
      <div className="mt-5">
        <ErrorState detail="The refget sequence request failed." title="Fetch failed" />
      </div>
    );
  }
  return false;
};

export default GeneSequenceErrors;
