import type { ReactElement, ReactNode } from "react";

const GeneSequenceDefinition = (props: { label: string; value: ReactNode }): ReactElement => (
  <>
    <dt className="text-text-muted">{props.label}</dt>
    <dd>{props.value}</dd>
  </>
);

export default GeneSequenceDefinition;
