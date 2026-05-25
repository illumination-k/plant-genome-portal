import type { ReactElement } from "react";

const GeneHeaderRow = (): ReactElement => (
  <tr>
    <th className="px-4 py-3 font-medium">Gene</th>
    <th className="px-4 py-3 font-medium">Assembly</th>
    <th className="px-4 py-3 font-medium">Location</th>
    <th className="px-4 py-3 font-medium">Strand</th>
    <th className="px-4 py-3 font-medium">Type</th>
  </tr>
);

export default GeneHeaderRow;
