import type { ReactElement } from "react";

const DatasetHeaderRow = (): ReactElement => (
  <tr>
    <th className="px-4 py-3 font-medium">Species</th>
    <th className="px-4 py-3 font-medium">Assembly</th>
    <th className="px-4 py-3 font-medium">Status</th>
  </tr>
);

export default DatasetHeaderRow;
