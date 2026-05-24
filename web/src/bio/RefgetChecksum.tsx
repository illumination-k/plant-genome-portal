import type { ReactElement } from "react";
import CopyButton from "@/ui/CopyButton";

const START_INDEX = 0;
const TRUNCATE_HEAD = 6;
const TRUNCATE_TAIL = 4;
const NEGATIVE_TAIL = -TRUNCATE_TAIL;

const truncate = (value: string): string => {
  if (value.length <= TRUNCATE_HEAD + TRUNCATE_TAIL) {
    return value;
  }
  return `${value.slice(START_INDEX, TRUNCATE_HEAD)}…${value.slice(NEGATIVE_TAIL)}`;
};

const RefgetChecksum = (props: { value: string }): ReactElement => (
  <span className="group inline-flex items-center gap-1.5">
    <span className="font-mono text-[12px] text-text-muted" title={props.value}>
      refget:{truncate(props.value)}
    </span>
    <CopyButton label="Copy refget checksum" value={props.value} />
  </span>
);

export default RefgetChecksum;
