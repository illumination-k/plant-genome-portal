import { keggPathwaysOptions } from "@/api/client/@tanstack/react-query.gen";
import type { KeggPathwaySummary } from "@/api/client/types.gen";
import { useQuery } from "@tanstack/react-query";
import type { ReactElement } from "react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router";
import PathwayCombobox from "@/features/kegg/components/PathwayCombobox";
import PathwayTable from "@/features/kegg/components/PathwayTable";
import EmptyState from "@/shared/ui/EmptyState";
import ErrorState from "@/shared/ui/ErrorState";
import Skeleton from "@/shared/ui/Skeleton";

const empty = 0;

const pathwayLabel = (pathway: KeggPathwaySummary): string =>
  `${pathway.pathway.id} ${pathway.pathway.name ?? ""}`.toLowerCase();

const filterPathways = (pathways: KeggPathwaySummary[], query: string): KeggPathwaySummary[] => {
  const normalized = query.trim().toLowerCase();
  if (normalized === "") {
    return pathways;
  }
  return pathways.filter((pathway) => pathwayLabel(pathway).includes(normalized));
};

const pathwayUrl = (pathwayId: string): string => `/kegg/pathway/${encodeURIComponent(pathwayId)}`;

const resolveSubmittedPathway = (
  pathways: KeggPathwaySummary[],
  query: string,
): string | undefined => {
  const trimmed = query.trim();
  const exact = pathways.find((pathway) => pathway.pathway.id === trimmed);
  return exact?.pathway.id ?? pathways[empty]?.pathway.id;
};

const PathwaysPage = (): ReactElement => {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const { data = [], error, isLoading } = useQuery(keggPathwaysOptions());
  const pathways = useMemo(() => filterPathways(data, query), [data, query]);

  const onSubmit = (value: string): void => {
    const pathwayId = resolveSubmittedPathway(pathways, value);
    if (pathwayId) {
      navigate(pathwayUrl(pathwayId));
    }
  };

  if (isLoading) {
    return <Skeleton size="panel" />;
  }
  if (error) {
    return <ErrorState detail={String(error)} title="Pathways could not be loaded" />;
  }
  if (data.length === empty) {
    return <EmptyState description="No KEGG pathway catalog is configured." title="No pathways" />;
  }

  return (
    <section className="grid grid-cols-12 gap-6">
      <header className="col-span-12">
        <h1 className="text-2xl font-semibold text-text">Pathways</h1>
        <p className="mt-2 max-w-[64ch] text-sm leading-6 text-text-muted">
          Browse KEGG pathways available in the current annotation catalog.
        </p>
      </header>
      <div className="col-span-12 lg:col-span-5">
        <PathwayCombobox
          onQueryChange={setQuery}
          onSubmit={onSubmit}
          pathways={data}
          query={query}
        />
      </div>
      <div className="col-span-12">
        <PathwayTable pathways={pathways} />
      </div>
    </section>
  );
};

export default PathwaysPage;
