import type { ReactElement } from "react";

const formatNumber = (pos: number): string => new Intl.NumberFormat("en-US").format(pos);

const CoordinateRange = (props: { chr: string; end: number; start: number }): ReactElement => (
  <span className="font-mono text-[13px] tabular-nums text-text-muted">
    {props.chr}:{formatNumber(props.start)}–{formatNumber(props.end)}
  </span>
);

export default CoordinateRange;
