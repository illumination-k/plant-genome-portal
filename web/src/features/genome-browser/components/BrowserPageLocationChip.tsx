import type { ReactElement } from "react";

const BrowserPageLocationChip = (props: { location: string }): ReactElement => (
  <span className="font-mono text-[12px] text-text-muted">{props.location}</span>
);

export default BrowserPageLocationChip;
