import type { ReactElement } from "react";

const LandingPopularRow = (props: { id: string; note: string }): ReactElement => (
  <li>
    <a
      className="flex items-baseline gap-3 px-3 py-3 hover:bg-surface-muted"
      href={`/genes/${props.id}`}
    >
      <span className="font-mono text-sm text-text">{props.id}</span>
      <span className="text-[13px] text-text-muted">{props.note}</span>
    </a>
  </li>
);

export default LandingPopularRow;
