import type { GeneRecord } from "@/api/client/types.gen";
import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactElement } from "react";
import geneRecordUtils from "@/lib/geneRecordUtils";
import GeneAttributesPanel from "@/components/GeneAttributesPanel";
import GeneBackLink from "@/components/GeneBackLink";
import GeneExonsPanel from "@/components/GeneExonsPanel";
import GeneHero from "@/components/GeneHero";
import GeneStatusGrid from "@/components/GeneStatusGrid";
import GeneTranscriptPanel from "@/components/GeneTranscriptPanel";

const GeneDetailState = (props: {
  geneId: string;
  geneQuery: UseQueryResult<GeneRecord, unknown>;
}): ReactElement => {
  if (props.geneId === "") {
    return <GeneStatusGrid detail="Open a gene from the genes page." title="Missing gene ID" />;
  }

  if (props.geneQuery.isLoading) {
    return <GeneStatusGrid detail={props.geneId} title="Loading gene" />;
  }

  if (props.geneQuery.error) {
    return (
      <GeneStatusGrid
        detail={geneRecordUtils.errorMessage(props.geneQuery.error)}
        title="Gene not found"
      />
    );
  }

  if (!props.geneQuery.data) {
    return <GeneStatusGrid detail={props.geneId} title="Gene not found" />;
  }

  return (
    <section className="grid grid-cols-12 gap-6">
      <GeneBackLink />
      <GeneHero geneRecord={props.geneQuery.data} />
      <GeneTranscriptPanel geneRecord={props.geneQuery.data} />
      <GeneAttributesPanel gene={props.geneQuery.data.gene} />
      <GeneExonsPanel exons={props.geneQuery.data.exons} />
    </section>
  );
};

export default GeneDetailState;
