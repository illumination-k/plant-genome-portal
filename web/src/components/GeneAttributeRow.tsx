import type { ReactElement } from "react";

const GeneAttributeRow = (props: { attributeKey: string; value: string }): ReactElement => (
  <div className="grid gap-2 py-3 sm:grid-cols-[10rem_1fr]">
    <dt className="font-medium text-text">{props.attributeKey}</dt>
    <dd className="break-words text-text-muted">{props.value}</dd>
  </div>
);

export default GeneAttributeRow;
