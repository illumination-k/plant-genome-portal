import type { ReactElement } from "react";
import BrowserPageHeaderText from "@/features/genome-browser/components/BrowserPageHeaderText";
import BrowserPageLocationChip from "@/features/genome-browser/components/BrowserPageLocationChip";

const renderLocation = (location: string): ReactElement | false => {
  if (location) {
    return <BrowserPageLocationChip location={location} />;
  }
  return false;
};

const BrowserPageHeader = (props: { location: string }): ReactElement => (
  <header className="flex flex-wrap items-baseline justify-between gap-3">
    <BrowserPageHeaderText />
    {renderLocation(props.location)}
  </header>
);

export default BrowserPageHeader;
