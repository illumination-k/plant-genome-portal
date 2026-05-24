import { geneSearchOptions } from "@/api/client/@tanstack/react-query.gen";
import { useQuery } from "@tanstack/react-query";
import { useSearchParams } from "react-router";
import type { GeneSearchData } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import GeneResultsPanel from "@/components/GeneResultsPanel";
import GeneSearchPanel from "@/components/GeneSearchPanel";

const geneSearchLimit = 25;
const queryParameterKey = "q";

const getQueryText = (searchParams: URLSearchParams): string => {
  const rawQuery = searchParams.get(queryParameterKey);
  return rawQuery ?? "";
};

const getGeneSearchQuery = (query: string): GeneSearchData["query"] => {
  if (query === "") {
    return { limit: geneSearchLimit };
  }

  return { [queryParameterKey]: query, limit: geneSearchLimit };
};

const GenesPage = (): ReactElement => {
  const [searchParams] = useSearchParams();
  const query = getQueryText(searchParams);
  const {
    data: genes = [],
    error,
    isFetching,
  } = useQuery(geneSearchOptions({ query: getGeneSearchQuery(query) }));

  return (
    <section className="grid grid-cols-12 gap-6">
      <GeneSearchPanel resultCount={genes.length} searchText={query} />
      <GeneResultsPanel error={error} genes={genes} isFetching={isFetching} />
    </section>
  );
};

export default GenesPage;
