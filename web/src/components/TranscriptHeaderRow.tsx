import type { ReactElement } from "react";

const TranscriptHeaderRow = (): ReactElement => (
  <tr>
    <th className="px-4 py-3 font-medium">Transcript</th>
    <th className="px-4 py-3 font-medium">Location</th>
    <th className="px-4 py-3 font-medium">Exons</th>
    <th className="px-4 py-3 font-medium">Type</th>
  </tr>
);

export default TranscriptHeaderRow;
