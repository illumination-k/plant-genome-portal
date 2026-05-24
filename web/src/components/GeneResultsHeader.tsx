import type { ReactElement } from "react";

const GeneResultsHeader = (props: { isFetching: boolean }): ReactElement => {
  if (props.isFetching) {
    return (
      <div className="flex items-center justify-between gap-4 border-b border-border-subtle px-6 py-4">
        <h3 className="text-base font-semibold">Matching genes</h3>
        <span className="text-sm text-text-muted">Loading</span>
      </div>
    );
  }

  return (
    <div className="border-b border-border-subtle px-6 py-4">
      <h3 className="text-base font-semibold">Matching genes</h3>
    </div>
  );
};

export default GeneResultsHeader;
