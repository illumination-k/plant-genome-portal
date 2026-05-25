import { Dialog } from "@base-ui/react/dialog";
import type { ReactElement } from "react";
import CommandPaletteContent from "@/app/command-palette/CommandPaletteContent";

const renderPopup = (
  <div className="fixed left-1/2 top-[15dvh] z-50 flex w-[min(640px,92vw)] -translate-x-1/2 flex-col overflow-hidden rounded-lg border border-border bg-surface-raised shadow-3 outline-none" />
);

const renderTitle = <h2 className="sr-only">Command palette</h2>;

const CommandPalettePopupBody = (props: {
  onClose: () => void;
  open: boolean;
}): ReactElement => (
  <Dialog.Popup render={renderPopup}>
    <Dialog.Title render={renderTitle} />
    <CommandPaletteContent onClose={props.onClose} open={props.open} />
  </Dialog.Popup>
);

export default CommandPalettePopupBody;
