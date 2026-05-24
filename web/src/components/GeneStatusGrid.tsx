import type { ReactElement } from "react";
import StatusMessage from "@/components/StatusMessage";

const GeneStatusGrid = (props: { detail: string; title: string }): ReactElement => (
  <section className="grid grid-cols-12 gap-6">
    <div className="col-span-12">
      <StatusMessage detail={props.detail} title={props.title} />
    </div>
  </section>
);

export default GeneStatusGrid;
