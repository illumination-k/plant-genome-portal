import type { ReactElement } from "react";

const ResultCountBadge = (props: { resultCount: number }): ReactElement => (
  <span className="rounded-md bg-emerald-100 px-3 py-2 text-sm font-semibold text-emerald-800">
    {props.resultCount} results
  </span>
);

export default ResultCountBadge;
