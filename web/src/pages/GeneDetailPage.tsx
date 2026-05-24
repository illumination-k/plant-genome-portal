import { geneOptions } from "@/api/client/@tanstack/react-query.gen";
import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router";
import type { ReactElement } from "react";
import GeneDetailState from "@/components/GeneDetailState";

const GeneDetailPage = (): ReactElement => {
  const params = useParams<{ geneId: string }>();
  const geneId = params.geneId ?? "";
  const geneQuery = useQuery(geneOptions({ path: { gene_id: geneId } }));

  return <GeneDetailState geneId={geneId} geneQuery={geneQuery} />;
};

export default GeneDetailPage;
