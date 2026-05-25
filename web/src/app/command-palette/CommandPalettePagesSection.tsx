import type { ReactElement } from "react";
import CommandPalettePageItem from "@/app/command-palette/CommandPalettePageItem";
import CommandPaletteSection from "@/app/command-palette/CommandPaletteSection";

type Page = { detail: string; label: string; to: string };

const EMPTY = 0;

const renderEmpty = (): ReactElement => (
  <p className="px-2 py-1.5 text-[12px] text-text-subtle">No pages</p>
);

const renderPages = (pages: Page[], onSelect: (to: string) => void): ReactElement => (
  <>
    {pages.map((page) => (
      <CommandPalettePageItem key={page.to} onSelect={onSelect} page={page} />
    ))}
  </>
);

const renderContent = (
  pages: Page[],
  onSelect: (to: string) => void,
): ReactElement => {
  if (pages.length === EMPTY) {
    return renderEmpty();
  }
  return renderPages(pages, onSelect);
};

const CommandPalettePagesSection = (props: {
  onSelect: (to: string) => void;
  pages: Page[];
}): ReactElement => (
  <CommandPaletteSection title="Pages">
    {renderContent(props.pages, props.onSelect)}
  </CommandPaletteSection>
);

export default CommandPalettePagesSection;
