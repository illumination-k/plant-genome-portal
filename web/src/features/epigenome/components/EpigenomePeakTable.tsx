/* oxlint-disable jsx-max-depth */
import type { PublicPeak } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import EpigenomePeakRow from "@/features/epigenome/components/EpigenomePeakRow";

const EpigenomePeakTable = (props: { peaks: PublicPeak[] }): ReactElement => (
  <table className="w-full text-left">
    <thead>
      <tr className="text-xs uppercase tracking-wide text-text-muted">
        <th className="px-3 py-2 font-medium">Peak</th>
        <th className="px-3 py-2 font-medium">Region</th>
        <th className="px-3 py-2 text-right font-medium">Signal</th>
        <th className="px-3 py-2 text-right font-medium">−log₁₀(q)</th>
      </tr>
    </thead>
    <tbody>
      {props.peaks.map((peak) => (
        <EpigenomePeakRow key={`${peak.sequenceName}-${peak.start}-${peak.name}`} peak={peak} />
      ))}
    </tbody>
  </table>
);

export default EpigenomePeakTable;
