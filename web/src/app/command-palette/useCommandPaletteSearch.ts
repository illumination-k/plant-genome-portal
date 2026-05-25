import { geneSearchOptions } from "@/api/client/@tanstack/react-query.gen";
import type { Gene } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";

const RESULT_LIMIT = 8;
const MIN_QUERY_LENGTH = 1;
const API_QUERY_KEY = "q";

type Page = { detail: string; label: string; to: string };

const defaultPages: Page[] = [
  { detail: "Search-first landing", label: "Search", to: "/" },
  { detail: "Browse the gene table", label: "Genes", to: "/genes" },
  { detail: "JBrowse genome browser", label: "Genome browser", to: "/browser" },
  { detail: "Assembly inventory", label: "Assemblies", to: "/datasets" },
];

const filterPages = (query: string): Page[] => {
  if (query === "") {
    return defaultPages;
  }
  const lower = query.toLowerCase();
  return defaultPages.filter((page) => page.label.toLowerCase().includes(lower));
};

type Result = {
  enabled: boolean;
  filteredPages: Page[];
  genes: Gene[];
};

const useCommandPaletteSearch = (query: string): Result => {
  const trimmed = query.trim();
  const enabled = trimmed.length >= MIN_QUERY_LENGTH;
  const baseOptions = geneSearchOptions({
    query: { [API_QUERY_KEY]: trimmed, limit: RESULT_LIMIT },
  });
  const { data: genes = [] } = useQuery({
    enabled,
    queryFn: baseOptions.queryFn,
    queryKey: baseOptions.queryKey,
  });
  return {
    enabled,
    filteredPages: filterPages(trimmed),
    genes,
  };
};

export default useCommandPaletteSearch;
