import type { ReactElement } from "react";

const ResultCountBadge = (props: { resultCount: number }): ReactElement => (
  <span className="rounded-md bg-primary-100 px-3 py-2 text-sm font-semibold text-primary-800">
    {props.resultCount} results
  </span>
);

export default ResultCountBadge;
