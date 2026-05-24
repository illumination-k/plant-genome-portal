import { ConfigurationSchema } from "@jbrowse/core/configuration";
import { BaseFeatureDataAdapter } from "@jbrowse/core/data_adapters/BaseAdapter";
import { ObservableCreate } from "@jbrowse/core/util/rxjs";
import SimpleFeature, { type Feature } from "@jbrowse/core/util/simpleFeature";
import type { Observable } from "rxjs";

type Region = {
  end: number;
  refName: string;
  start: number;
};

type FeatureResponse = {
  end: number;
  name: string;
  refName: string;
  start: number;
  strand: number;
  type: string;
  uniqueId: string;
};

// oxlint-disable-next-line new-cap -- JBrowse factory returns a schema type
const configSchema = ConfigurationSchema(
  "PgpFeaturesAdapter",
  {
    featuresUrl: {
      defaultValue: "",
      description: "Plant Genome Portal features endpoint",
      type: "string",
    },
  },
  { explicitlyTyped: true },
);

const fetchFeatures = async (
  featuresUrl: string,
  region: Region,
): Promise<FeatureResponse[]> => {
  const params = new URLSearchParams({
    end: String(region.end),
    refName: region.refName,
    start: String(region.start),
  });
  const response = await fetch(`${featuresUrl}?${params.toString()}`);
  if (!response.ok) {
    throw new Error(`features request failed: ${response.status}`);
  }
  return (await response.json()) as FeatureResponse[];
};

const toSimpleFeature = (feature: FeatureResponse): SimpleFeature =>
  new SimpleFeature({
    end: feature.end,
    name: feature.name,
    refName: feature.refName,
    start: feature.start,
    strand: feature.strand,
    type: feature.type,
    uniqueId: feature.uniqueId,
  });

class PgpFeaturesAdapter extends BaseFeatureDataAdapter {
  // oxlint-disable-next-line class-methods-use-this, require-await -- JBrowse adapter contract
  public async getRefNames(): Promise<string[]> {
    return [];
  }

  public getFeatures(region: Region): Observable<Feature> {
    const featuresUrl = String(this.getConf("featuresUrl"));
    // oxlint-disable-next-line new-cap -- JBrowse factory returns an Observable
    return ObservableCreate<Feature>(async (observer) => {
      if (featuresUrl === "") {
        observer.complete();
        return;
      }
      try {
        const features = await fetchFeatures(featuresUrl, region);
        for (const feature of features) {
          observer.next(toSimpleFeature(feature));
        }
        observer.complete();
      } catch (error) {
        observer.error(error);
      }
    });
  }
}

const pgpFeaturesAdapter = {
  Adapter: PgpFeaturesAdapter,
  configSchema,
};

export default pgpFeaturesAdapter;
