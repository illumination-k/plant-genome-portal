import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import CommandPaletteGenesSection from "@/ui/CommandPaletteGenesSection";
import CommandPalettePagesSection from "@/ui/CommandPalettePagesSection";

type Page = { detail: string; label: string; to: string };

const CommandPaletteBody = (props: {
  enabled: boolean;
  genes: Gene[];
  onSelect: (to: string) => void;
  pages: Page[];
}): ReactElement => (
  <div className="max-h-[60dvh] overflow-y-auto p-2">
    <CommandPalettePagesSection onSelect={props.onSelect} pages={props.pages} />
    <CommandPaletteGenesSection
      enabled={props.enabled}
      genes={props.genes}
      onSelect={props.onSelect}
    />
  </div>
);

export default CommandPaletteBody;
