import type { Transcript } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import geneFormat from "@/lib/geneFormat";

const TranscriptRow = (props: { exonCount: number; transcript: Transcript }): ReactElement => (
  <tr>
    <td className="px-4 py-3 font-medium text-zinc-900">{props.transcript.id}</td>
    <td className="px-4 py-3 text-zinc-600">
      {geneFormat.formatLocation(props.transcript.sequence_name, props.transcript.region)}
    </td>
    <td className="px-4 py-3 text-zinc-600">{props.exonCount}</td>
    <td className="px-4 py-3 text-zinc-600">{props.transcript.feature_type}</td>
  </tr>
);

export default TranscriptRow;
