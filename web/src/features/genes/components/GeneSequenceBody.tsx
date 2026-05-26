import type { ReactElement } from "react";

const sequenceLineLength = 60;

const wrapSequence = (sequence: string): string => {
  const lines: string[] = [];
  for (let index = 0; index < sequence.length; index += sequenceLineLength) {
    lines.push(sequence.slice(index, index + sequenceLineLength));
  }
  return lines.join("\n");
};

const GeneSequenceBody = (props: { isLoading: boolean; sequence: string }): ReactElement => {
  if (props.isLoading) {
    return <p className="mt-6 text-sm text-text-muted">Loading sequence.</p>;
  }
  if (!props.sequence) {
    return <p className="mt-6 text-sm text-text-muted">No sequence loaded.</p>;
  }
  return (
    <pre className="mt-5 max-h-[28rem] overflow-auto rounded-md border border-border-subtle bg-surface-muted p-4 font-mono text-[12px] leading-5 text-text">
      {wrapSequence(props.sequence)}
    </pre>
  );
};

export default GeneSequenceBody;
