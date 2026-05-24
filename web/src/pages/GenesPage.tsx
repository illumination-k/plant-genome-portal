import { geneSearchOptions } from "@/api/client/@tanstack/react-query.gen";
import { useQuery } from "@tanstack/react-query";
import type { GeneSearchData } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { minLength, pipe, string } from "valibot";
import GeneResultsPanel from "@/components/GeneResultsPanel";
import GeneSearchPanel from "@/components/GeneSearchPanel";
import useValidatedSearchParam from "@/lib/useValidatedSearchParam";

const geneSearchLimit = 25;
const queryParameterKey = "q";

const MIN_QUERY_LENGTH = 1;
const querySchema = pipe(string(), minLength(MIN_QUERY_LENGTH));

const buildSearchQuery = (query: string): GeneSearchData["query"] => {
  if (query === "") {
    return { limit: geneSearchLimit };
  }
  return { [queryParameterKey]: query, limit: geneSearchLimit };
};

const GenesPage = (): ReactElement => {
  const query = useValidatedSearchParam(queryParameterKey, querySchema, "");
  const {
    data: genes = [],
    error,
    isFetching,
  } = useQuery(geneSearchOptions({ query: buildSearchQuery(query) }));

  return (
    <section className="grid grid-cols-12 gap-6">
      <GeneSearchPanel resultCount={genes.length} searchText={query} />
      <GeneResultsPanel error={error} genes={genes} isFetching={isFetching} />
    </section>
  );
};

export default GenesPage;
