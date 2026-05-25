import { geneOptions } from "@/api/client/@tanstack/react-query.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import { minLength, pipe, string } from "valibot";
import GeneDetailState from "@/features/genes/components/GeneDetailState";
import useValidatedParam from "@/shared/lib/useValidatedParam";

const MIN_GENE_ID_LENGTH = 1;
const geneIdSchema = pipe(string(), minLength(MIN_GENE_ID_LENGTH));

const GeneDetailPage = (): ReactElement => {
  const geneId = useValidatedParam("geneId", geneIdSchema, "");
  const geneQuery = useQuery(geneOptions({ path: { gene_id: geneId } }));

  return <GeneDetailState geneId={geneId} geneQuery={geneQuery} />;
};

export default GeneDetailPage;
