import type { ReactElement } from "react";

const GeneResultsHeader = (props: { isFetching: boolean }): ReactElement => {
  if (props.isFetching) {
    return (
      <div className="flex items-center justify-between gap-4 border-b border-zinc-200 px-6 py-4">
        <h3 className="text-base font-semibold">Matching genes</h3>
        <span className="text-sm text-zinc-500">Loading</span>
      </div>
    );
  }

  return (
    <div className="border-b border-zinc-200 px-6 py-4">
      <h3 className="text-base font-semibold">Matching genes</h3>
    </div>
  );
};

export default GeneResultsHeader;
