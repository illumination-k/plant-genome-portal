import type { Strand } from "@/api/client/types.gen";
import type { ReactElement } from "react";

const tones = {
  forward: {
    className: "border-strand-forward/40 bg-strand-forward/10 text-strand-forward",
    label: "+ strand",
    symbol: "+",
  },
  reverse: {
    className: "border-strand-reverse/40 bg-strand-reverse/10 text-strand-reverse",
    label: "− strand",
    symbol: "−",
  },
  unknown: {
    className: "border-border bg-surface-muted text-text-muted",
    label: "unknown strand",
    symbol: "·",
  },
};

const StrandBadge = (props: { strand: Strand }): ReactElement => {
  const tone = tones[props.strand];
  return (
    <span
      aria-label={tone.label}
      className={`inline-flex h-[18px] min-w-[26px] items-center justify-center rounded-full border px-1.5 font-mono text-[12px] font-semibold ${tone.className}`}
      title={tone.label}
    >
      {tone.symbol}
    </span>
  );
};

export default StrandBadge;
