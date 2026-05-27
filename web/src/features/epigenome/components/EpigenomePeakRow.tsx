/* oxlint-disable no-magic-numbers */
import type { PublicPeak } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const PRECISION = 2;
const QVALUE_UNAVAILABLE = -1;

const formatQValue = (qValue: number): string => {
  if (qValue === QVALUE_UNAVAILABLE || qValue < 0) {
    return "—";
  }
  return qValue.toFixed(PRECISION);
};

const EpigenomePeakRow = (props: { peak: PublicPeak }): ReactElement => (
  <tr className="border-t border-border-subtle text-sm">
    <td className="px-3 py-2 font-mono text-xs text-text-muted">{props.peak.name}</td>
    <td className="px-3 py-2 font-mono text-xs text-text-muted">
      {props.peak.sequenceName}:{props.peak.start.toLocaleString()}-
      {props.peak.end.toLocaleString()}
    </td>
    <td className="px-3 py-2 text-right tabular-nums text-text">
      {props.peak.signalValue.toFixed(PRECISION)}
    </td>
    <td className="px-3 py-2 text-right tabular-nums text-text-muted">
      {formatQValue(props.peak.qValue)}
    </td>
  </tr>
);

export default EpigenomePeakRow;
