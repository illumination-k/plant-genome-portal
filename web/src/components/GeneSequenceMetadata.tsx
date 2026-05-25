import type { ReactElement } from "react";
import GeneSequenceDefinition from "@/components/GeneSequenceDefinition";

const formatNumber = (pos: number): string => pos.toLocaleString("en-US");

const renderChromosome = (chr: string): ReactElement => (
  <span className="font-mono text-text">{chr}</span>
);

const renderRange = (start: number, end: number, length: number): ReactElement => (
  <span className="font-mono tabular-nums text-text">
    {formatNumber(start)}-{formatNumber(end)} · {formatNumber(length)} bp
  </span>
);

const renderRefget = (): ReactElement => (
  <a
    className="font-mono text-[13px] text-primary-800 hover:text-primary-900 hover:underline"
    href="/sequence/service-info"
    rel="noreferrer"
    target="_blank"
  >
    /sequence/service-info
  </a>
);

const renderEndpoint = (endpointUrl: string): ReactElement => (
  <a
    className="break-all font-mono text-[13px] text-primary-800 hover:text-primary-900 hover:underline"
    href={endpointUrl}
    rel="noreferrer"
    target="_blank"
  >
    {endpointUrl}
  </a>
);

const GeneSequenceMetadata = (props: {
  chr: string;
  end: number;
  endpointUrl: string | undefined;
  length: number;
  start: number;
}): ReactElement => (
  <dl className="mt-4 grid grid-cols-1 gap-3 text-[13px] sm:grid-cols-[10rem_1fr]">
    <GeneSequenceDefinition label="Chromosome" value={renderChromosome(props.chr)} />
    <GeneSequenceDefinition
      label="Region (1-based closed)"
      value={renderRange(props.start, props.end, props.length)}
    />
    <GeneSequenceDefinition label="refget service" value={renderRefget()} />
    {props.endpointUrl && (
      <GeneSequenceDefinition
        label="refget endpoint"
        value={renderEndpoint(props.endpointUrl)}
      />
    )}
  </dl>
);

export default GeneSequenceMetadata;
