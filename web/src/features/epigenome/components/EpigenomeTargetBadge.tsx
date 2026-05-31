import type { ReactElement } from "react";

const EpigenomeTargetBadge = (props: {
  target: string | null | undefined;
}): ReactElement | undefined => {
  if (props.target === null || props.target === undefined || props.target === "") {
    return undefined;
  }
  return (
    <span className="inline-flex items-center rounded-md border border-accent/30 bg-accent/10 px-2 py-0.5 font-mono text-xs text-accent">
      {props.target}
    </span>
  );
};

export default EpigenomeTargetBadge;
