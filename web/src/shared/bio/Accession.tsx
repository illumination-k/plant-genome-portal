import type { ReactElement } from "react";
import CopyButton from "@/shared/ui/CopyButton";

const Accession = (props: { value: string }): ReactElement => (
  <span className="group inline-flex items-center gap-1.5">
    <span className="font-mono text-[13px] text-text">{props.value}</span>
    <CopyButton label="Copy accession" value={props.value} />
  </span>
);

export default Accession;
