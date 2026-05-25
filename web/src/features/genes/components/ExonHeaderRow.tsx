import type { ReactElement } from "react";

const ExonHeaderRow = (): ReactElement => (
  <tr>
    <th className="px-4 py-3 font-medium">Transcript</th>
    <th className="px-4 py-3 font-medium">Sequence</th>
    <th className="px-4 py-3 font-medium">Region</th>
    <th className="px-4 py-3 font-medium">Strand</th>
  </tr>
);

export default ExonHeaderRow;
