import Plugin from "@jbrowse/core/Plugin";
import type PluginManager from "@jbrowse/core/PluginManager";
import AdapterType from "@jbrowse/core/pluggableElementTypes/AdapterType";
import pgpFeaturesAdapter from "@/components/GenomeBrowser/pgpFeaturesAdapter";

const buildAdapterType = (): AdapterType =>
  new AdapterType({
    AdapterClass: pgpFeaturesAdapter.Adapter,
    configSchema: pgpFeaturesAdapter.configSchema,
    name: "PgpFeaturesAdapter",
  });

export default class PgpFeaturesPlugin extends Plugin {
  public name = "PgpFeaturesPlugin";

  // oxlint-disable-next-line class-methods-use-this -- JBrowse plugin contract
  public install(pluginManager: PluginManager): void {
    pluginManager.addAdapterType(buildAdapterType);
  }
}
