import type { Gene } from "@/api/client/types.gen";

const oneBasedOffset = 1;

const formatPosition = (position: number): string =>
  new Intl.NumberFormat("en-US").format(position);

const formatLocation = (sequenceName: string, region: Gene["region"]): string =>
  `${sequenceName}:${formatPosition(region.start + oneBasedOffset)}-${formatPosition(region.end)}`;

const formatRegion = (gene: Gene): string =>
  `${formatPosition(gene.region.start + oneBasedOffset)}-${formatPosition(gene.region.end)}`;

const formatStrand = (strand: Gene["strand"]): string => {
  if (strand === "forward") {
    return "+";
  }

  if (strand === "reverse") {
    return "-";
  }

  return ".";
};

const getErrorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }

  return "The API request failed.";
};

const geneFormat = {
  formatLocation,
  formatPosition,
  formatRegion,
  formatStrand,
  getErrorMessage,
};

export default geneFormat;
