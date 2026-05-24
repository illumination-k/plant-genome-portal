import type { ReactElement } from "react";
import { minLength, pipe, string } from "valibot";
import BrowserPageHeader from "@/components/BrowserPageHeader";
import GenomeBrowser from "@/components/GenomeBrowser";
import useValidatedSearchParam from "@/lib/useValidatedSearchParam";

const MIN_LOC_LENGTH = 1;
const locationSchema = pipe(string(), minLength(MIN_LOC_LENGTH));

const BrowserPage = (): ReactElement => {
  const location = useValidatedSearchParam("loc", locationSchema, "");

  return (
    <section className="flex flex-col gap-4">
      <BrowserPageHeader location={location} />
      <div className="overflow-hidden rounded-lg border border-border-subtle bg-surface">
        <GenomeBrowser location={location} />
      </div>
    </section>
  );
};

export default BrowserPage;
