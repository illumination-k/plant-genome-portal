import type { ReactElement } from "react";
import Skeleton from "@/shared/ui/Skeleton";

const GeneDetailLoading = (props: { geneId: string }): ReactElement => (
  <section className="flex flex-col gap-6">
    <Skeleton size="caption" />
    <div className="flex flex-col gap-2">
      <Skeleton size="title" />
      <Skeleton size="body" />
    </div>
    <Skeleton size="row" />
    <Skeleton size="panel" />
    <p className="text-[12px] text-text-subtle">
      Loading <span className="font-mono">{props.geneId}</span>…
    </p>
  </section>
);

export default GeneDetailLoading;
