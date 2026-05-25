import type { ReactElement, ReactNode } from "react";

const Sci = (props: { children: ReactNode }): ReactElement => (
  <span className="italic">{props.children}</span>
);

export default Sci;
