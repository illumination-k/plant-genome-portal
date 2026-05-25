import type { ReactElement } from "react";

const sizes = {
  body: "h-4 w-96",
  caption: "h-4 w-24",
  panel: "h-[320px] w-full",
  row: "h-10 w-full",
  title: "h-8 w-64",
};

type Size = keyof typeof sizes;

const Skeleton = (props: { size: Size }): ReactElement => (
  <span
    aria-hidden="true"
    className={`inline-block animate-pulse rounded bg-surface-muted ${sizes[props.size]}`}
  />
);

export default Skeleton;
