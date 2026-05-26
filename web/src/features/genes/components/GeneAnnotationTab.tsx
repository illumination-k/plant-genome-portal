import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useMemo } from "react";
import GeneAnnotationGroupCard from "@/features/genes/components/GeneAnnotationGroupCard";
import GeneKeggCard from "@/features/genes/components/GeneKeggCard";
import annotationEntries from "@/shared/lib/annotationEntries";

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
      <div className="col-span-12">
        <GeneKeggCard geneId={props.gene.id} />
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
