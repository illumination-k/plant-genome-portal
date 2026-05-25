import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import Accession from "@/shared/bio/Accession";
import CoordinateRange from "@/shared/bio/CoordinateRange";
import GeneSymbolLine from "@/shared/bio/GeneSymbolLine";
import Sci from "@/shared/bio/Sci";
import StrandBadge from "@/shared/bio/StrandBadge";

const oneBasedOffset = 1;

const renderSymbol = (symbol: string): ReactElement | false => {
  if (symbol) {
    return <GeneSymbolLine symbol={symbol} />;
  }
  return false;
};

const GeneHeader = (props: { gene: Gene }): ReactElement => {
  const { region } = props.gene;
  const start = region.start + oneBasedOffset;
  const { end } = region;
  const length = end - region.start;
  const symbol = props.gene.symbol ?? "";

  return (
    <header className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="font-mono text-[28px] font-bold tracking-tight text-text">
          {props.gene.id}
        </h1>
        <StrandBadge strand={props.gene.strand} />
        <CoordinateRange chr={props.gene.sequence_name} end={end} start={start} />
        <span className="font-mono text-[13px] tabular-nums text-text-muted">
          {length.toLocaleString("en-US")} bp
        </span>
      </div>
      {renderSymbol(symbol)}
      <p className="text-[13px] text-text-subtle">
        <Sci>Marchantia polymorpha</Sci> · <Accession value={props.gene.assembly_accession} />
      </p>
    </header>
  );
};

export default GeneHeader;
