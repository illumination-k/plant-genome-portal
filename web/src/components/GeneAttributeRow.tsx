import type { ReactElement } from "react";

const GeneAttributeRow = (props: { attributeKey: string; value: string }): ReactElement => (
  <div className="grid gap-2 py-3 sm:grid-cols-[10rem_1fr]">
    <dt className="font-medium text-zinc-700">{props.attributeKey}</dt>
    <dd className="break-words text-zinc-600">{props.value}</dd>
  </div>
);

export default GeneAttributeRow;
