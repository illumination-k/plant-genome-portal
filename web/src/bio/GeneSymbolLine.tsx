import type { ReactElement } from "react";

const GeneSymbolLine = (props: { symbol: string }): ReactElement => (
  <p className="text-[15px] text-text-muted">
    <span className="font-semibold text-text">{props.symbol}</span>
  </p>
);

export default GeneSymbolLine;
