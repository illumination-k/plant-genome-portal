import type { ReactElement } from "react";
import type datasetExport from "@/data/datasets";

type Dataset = (typeof datasetExport.datasets)[number];

const DatasetRow = (props: { dataset: Dataset }): ReactElement => (
  <tr>
    <td className="px-4 py-3 font-medium text-text">{props.dataset.species}</td>
    <td className="px-4 py-3 text-text-muted">{props.dataset.assembly}</td>
    <td className="px-4 py-3">
      <span className="rounded-md bg-sky-100 px-2 py-1 text-xs font-semibold text-sky-800">
        {props.dataset.status}
      </span>
    </td>
  </tr>
);

export default DatasetRow;
