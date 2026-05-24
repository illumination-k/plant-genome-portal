import {
  jbrowseConfigOptions,
  jbrowseDefaultConfigOptions,
} from "@/api/client/@tanstack/react-query.gen";
import type { JBrowseRootConfig } from "@/api/client/types.gen";
import PgpFeaturesPlugin from "@/components/GenomeBrowser/pgpFeaturesPlugin";
import { JBrowseLinearGenomeView, createViewState } from "@jbrowse/react-linear-genome-view2";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { ReactElement } from "react";

type GenomeBrowserProps = {
  accession?: string;
  location?: string;
};

type ViewState = ReturnType<typeof createViewState>;

type ConfigQuery = UseQueryResult<JBrowseRootConfig, unknown>;

const minBrowserHeightPx = 360;
const minHeightStyle = { minHeight: minBrowserHeightPx };
const emptyAccession = "";

const buildTrackId = (accession: string): string => `${accession}-pgp-genes`;

const buildViewState = (
  config: JBrowseRootConfig,
  location: string | undefined,
): ViewState => {
  const { plantGenomePortal: portal, assemblies, defaultSession: session } = config;
  const [assembly] = assemblies;
  const trackId = buildTrackId(portal.assemblyAccession);
  const initialLoc = location ?? session.view.init.loc;

  return createViewState({
    assembly,
    defaultSession: {
      name: session.name,
      view: {
        id: session.view.id,
        init: {
          assembly: portal.assemblyAccession,
          loc: initialLoc,
          tracks: [trackId],
        },
        type: session.view.type,
      },
    },
    location: initialLoc,
    plugins: [PgpFeaturesPlugin],
    tracks: [
      {
        adapter: {
          featuresUrl: portal.featuresUrl,
          type: "PgpFeaturesAdapter",
        },
        assemblyNames: [portal.assemblyAccession],
        name: "Genes",
        trackId,
        type: "FeatureTrack",
      },
    ],
  });
};

const useConfigQuery = (accession: string | undefined): ConfigQuery => {
  const scoped = typeof accession === "string" && accession !== emptyAccession;
  const defaultBase = jbrowseDefaultConfigOptions();
  const defaultQuery = useQuery({
    enabled: !scoped,
    queryFn: defaultBase.queryFn,
    queryKey: defaultBase.queryKey,
  });
  const accessionBase = jbrowseConfigOptions({
    path: { accession: accession ?? emptyAccession },
  });
  const accessionQuery = useQuery({
    enabled: scoped,
    queryFn: accessionBase.queryFn,
    queryKey: accessionBase.queryKey,
  });
  if (scoped) {
    return accessionQuery;
  }
  return defaultQuery;
};

const GenomeBrowser = (props: GenomeBrowserProps): ReactElement => {
  const configQuery = useConfigQuery(props.accession);
  const config = configQuery.data;
  const [viewState, setViewState] = useState<ViewState | undefined>();

  useEffect(() => {
    if (config) {
      setViewState(buildViewState(config, props.location));
    }
  }, [config, props.location]);

  if (configQuery.isLoading) {
    return (
      <div
        className="flex items-center justify-center rounded-md bg-zinc-50 text-sm text-zinc-500"
        style={minHeightStyle}
      >
        Loading genome browser…
      </div>
    );
  }

  if (configQuery.error || !viewState) {
    return (
      <div
        className="flex items-center justify-center rounded-md bg-rose-50 text-sm text-rose-700"
        style={minHeightStyle}
      >
        Genome browser unavailable.
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-md border border-zinc-200">
      <JBrowseLinearGenomeView viewState={viewState} />
    </div>
  );
};

export default GenomeBrowser;
