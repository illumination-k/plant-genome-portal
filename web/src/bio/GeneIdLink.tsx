import type { ReactElement } from "react";

const GeneIdLink = (props: { geneId: string }): ReactElement => (
  <a
    className="font-mono text-[13px] font-medium text-primary-800 hover:text-primary-900 hover:underline"
    href={`/genes/${props.geneId}`}
  >
    {props.geneId}
  </a>
);

export default GeneIdLink;
