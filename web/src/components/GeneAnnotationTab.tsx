import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useMemo } from "react";
import GeneAnnotationGroupCard from "@/components/GeneAnnotationGroupCard";
import annotationEntries from "@/lib/annotationEntries";

const GeneAnnotationTab = (props: { gene: Gene }): ReactElement => {
  const entries = useMemo(() => annotationEntries.buildEntries(props.gene), [props.gene]);

  return (
    <section className="grid grid-cols-12 gap-6">
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.go} label="GO terms" />
      </div>
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.pfam} label="Pfam" />
      </div>
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.interPro} label="InterPro" />
      </div>
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.kegg} label="KEGG" />
      </div>
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.ncbiFam} label="NCBIfam" />
      </div>
      <div className="col-span-12 lg:col-span-6">
        <GeneAnnotationGroupCard entries={entries.kog} label="KOG" />
      </div>
    </section>
  );
};

export default GeneAnnotationTab;
