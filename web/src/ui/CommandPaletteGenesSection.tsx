import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import CommandPaletteGeneItem from "@/ui/CommandPaletteGeneItem";
import CommandPaletteSection from "@/ui/CommandPaletteSection";

const EMPTY = 0;

const renderHint = (): ReactElement => (
  <p className="px-2 py-1.5 text-[12px] text-text-subtle">Type to search genes…</p>
);

const renderEmpty = (): ReactElement => (
  <p className="px-2 py-1.5 text-[12px] text-text-subtle">No matching genes</p>
);

const renderGenes = (genes: Gene[], onSelect: (to: string) => void): ReactElement => (
  <>
    {genes.map((gene) => (
      <CommandPaletteGeneItem gene={gene} key={gene.id} onSelect={onSelect} />
    ))}
  </>
);

const renderContent = (
  enabled: boolean,
  genes: Gene[],
  onSelect: (to: string) => void,
): ReactElement => {
  if (!enabled) {
    return renderHint();
  }
  if (genes.length === EMPTY) {
    return renderEmpty();
  }
  return renderGenes(genes, onSelect);
};

const CommandPaletteGenesSection = (props: {
  enabled: boolean;
  genes: Gene[];
  onSelect: (to: string) => void;
}): ReactElement => (
  <CommandPaletteSection title="Genes">
    {renderContent(props.enabled, props.genes, props.onSelect)}
  </CommandPaletteSection>
);

export default CommandPaletteGenesSection;
