import type { ReactElement } from "react";
import GeneSequenceDefinition from "@/components/GeneSequenceDefinition";

const formatNumber = (pos: number): string => pos.toLocaleString("en-US");

const renderChromosome = (chr: string): ReactElement => (
  <span className="font-mono text-text">{chr}</span>
);

const renderRange = (start: number, end: number, length: number): ReactElement => (
  <span className="font-mono tabular-nums text-text">
    {formatNumber(start)}–{formatNumber(end)} · {formatNumber(length)} bp
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

const GeneSequenceCard = (props: {
  chr: string;
  end: number;
  length: number;
  start: number;
}): ReactElement => (
  <div className="col-span-12 rounded-lg border border-border-subtle bg-surface p-6">
    <h3 className="text-base font-semibold text-text">Reference sequence</h3>
    <p className="mt-2 text-sm text-text-muted">
      Fetch this gene&apos;s reference sequence via the GA4GH refget endpoint. The API exposes
      per-chromosome FASTA indexed by SHA-512/24 base64url checksum.
    </p>
    <dl className="mt-4 grid grid-cols-1 gap-3 text-[13px] sm:grid-cols-[10rem_1fr]">
      <GeneSequenceDefinition label="Chromosome" value={renderChromosome(props.chr)} />
      <GeneSequenceDefinition
        label="Region (1-based closed)"
        value={renderRange(props.start, props.end, props.length)}
      />
      <GeneSequenceDefinition label="refget service" value={renderRefget()} />
    </dl>
    <p className="mt-6 text-[12px] text-text-subtle">
      A full sequence viewer (60-char block view, copy + download) will land alongside the refget
      proxy in a future iteration.
    </p>
  </div>
);

export default GeneSequenceCard;
